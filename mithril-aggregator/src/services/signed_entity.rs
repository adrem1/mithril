//! ## SignedEntityService
//!
//! This service is responsible of dealing with [SignedEntity] type.
//! It creates [Artifact] that can be accessed by clients.
use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use slog_scope::info;
use std::sync::Arc;

use mithril_common::{
    entities::{
        Beacon, CardanoTransactionsCommitment, Certificate, Epoch, MithrilStakeDistribution,
        SignedEntity, SignedEntityType, SignedEntityTypeDiscriminants, Snapshot,
    },
    signable_builder::Artifact,
    StdResult,
};

use crate::{
    artifact_builder::ArtifactBuilder,
    database::provider::{SignedEntityRecord, SignedEntityStorer},
};

#[cfg(test)]
use mockall::automock;

/// ArtifactBuilder Service trait
#[cfg_attr(test, automock)]
#[async_trait]
pub trait SignedEntityService: Send + Sync {
    /// Create artifact for a signed entity type and a certificate
    async fn create_artifact(
        &self,
        signed_entity_type: SignedEntityType,
        certificate: &Certificate,
    ) -> StdResult<()>;

    /// Return a list of signed snapshots order by creation date descending.
    async fn get_last_signed_snapshots(
        &self,
        total: usize,
    ) -> StdResult<Vec<SignedEntity<Snapshot>>>;

    /// Return a list of signed Mithril stake distribution order by creation
    /// date descending.
    async fn get_last_signed_mithril_stake_distributions(
        &self,
        total: usize,
    ) -> StdResult<Vec<SignedEntity<MithrilStakeDistribution>>>;

    /// Return a signed snapshot
    async fn get_signed_snapshot_by_id(
        &self,
        signed_entity_id: &str,
    ) -> StdResult<Option<SignedEntity<Snapshot>>>;

    /// Return a signed Mithril stake distribution
    async fn get_signed_mithril_stake_distribution_by_id(
        &self,
        signed_entity_id: &str,
    ) -> StdResult<Option<SignedEntity<MithrilStakeDistribution>>>;
}

/// Mithril ArtifactBuilder Service
pub struct MithrilSignedEntityService {
    signed_entity_storer: Arc<dyn SignedEntityStorer>,
    mithril_stake_distribution_artifact_builder:
        Arc<dyn ArtifactBuilder<Epoch, MithrilStakeDistribution>>,
    cardano_immutable_files_full_artifact_builder: Arc<dyn ArtifactBuilder<Beacon, Snapshot>>,
    cardano_transactions_artifact_builder:
        Arc<dyn ArtifactBuilder<Beacon, CardanoTransactionsCommitment>>,
}

impl MithrilSignedEntityService {
    /// MithrilSignedEntityService factory
    pub fn new(
        signed_entity_storer: Arc<dyn SignedEntityStorer>,
        mithril_stake_distribution_artifact_builder: Arc<
            dyn ArtifactBuilder<Epoch, MithrilStakeDistribution>,
        >,
        cardano_immutable_files_full_artifact_builder: Arc<dyn ArtifactBuilder<Beacon, Snapshot>>,
        cardano_transactions_artifact_builder: Arc<
            dyn ArtifactBuilder<Beacon, CardanoTransactionsCommitment>,
        >,
    ) -> Self {
        Self {
            signed_entity_storer,
            mithril_stake_distribution_artifact_builder,
            cardano_immutable_files_full_artifact_builder,
            cardano_transactions_artifact_builder,
        }
    }

    /// Compute artifact from signed entity type
    async fn compute_artifact(
        &self,
        signed_entity_type: SignedEntityType,
        certificate: &Certificate,
    ) -> StdResult<Option<Arc<dyn Artifact>>> {
        match signed_entity_type.clone() {
            SignedEntityType::MithrilStakeDistribution(epoch) => Ok(Some(Arc::new(
                self.mithril_stake_distribution_artifact_builder
                    .compute_artifact(epoch, certificate)
                    .await
                    .with_context(|| {
                        format!(
                            "Signed Entity Service can not compute artifact for entity type: '{signed_entity_type}'"
                        )
                    })?,
            ))),
            SignedEntityType::CardanoImmutableFilesFull(beacon) => Ok(Some(Arc::new(
                self.cardano_immutable_files_full_artifact_builder
                    .compute_artifact(beacon.clone(), certificate)
                    .await
                    .with_context(|| {
                        format!(
                            "Signed Entity Service can not compute artifact for entity type: '{signed_entity_type}'"
                        )
                    })?,
            ))),
            SignedEntityType::CardanoStakeDistribution(_) => todo!(),
            SignedEntityType::CardanoTransactions(beacon) => Ok(Some(Arc::new(
                self.cardano_transactions_artifact_builder
                    .compute_artifact(beacon.clone(), certificate)
                    .await
                    .with_context(|| {
                        format!(
                            "Signed Entity Service can not compute artifact for entity type: '{signed_entity_type}'"
                        )
                    })?,
            ))),
        }
    }
}

#[async_trait]
impl SignedEntityService for MithrilSignedEntityService {
    async fn create_artifact(
        &self,
        signed_entity_type: SignedEntityType,
        certificate: &Certificate,
    ) -> StdResult<()> {
        info!(
            "MithrilSignedEntityService::create_artifact";
            "signed_entity_type" => ?signed_entity_type,
            "certificate_hash" => &certificate.hash
        );

        let mut remaining_retries = 2;
        let artifact = loop {
            remaining_retries -= 1;

            match self
                .compute_artifact(signed_entity_type.clone(), certificate)
                .await
            {
                Err(error) if remaining_retries == 0 => break Err(error),
                Err(_error) => (),
                Ok(Some(artifact)) => break Ok(artifact),
                Ok(None) => return Ok(()),
            };
        }?;

        let signed_entity = SignedEntityRecord {
            signed_entity_id: artifact.get_id(),
            signed_entity_type: signed_entity_type.clone(),
            certificate_id: certificate.hash.clone(),
            artifact: serde_json::to_string(&artifact)?,
            created_at: Utc::now(),
        };

        self.signed_entity_storer
            .store_signed_entity(&signed_entity)
            .await
            .with_context(|| {
                format!(
                    "Signed Entity Service can not store signed entity with type: '{signed_entity_type}'"
                )
            })?;

        Ok(())
    }

    async fn get_last_signed_snapshots(
        &self,
        total: usize,
    ) -> StdResult<Vec<SignedEntity<Snapshot>>> {
        let signed_entities_records = self
            .signed_entity_storer
            .get_last_signed_entities_by_type(
                &SignedEntityTypeDiscriminants::CardanoImmutableFilesFull,
                total,
            )
            .await
            .with_context(|| {
                format!(
                    "Signed Entity Service can not get last signed entities with type: '{:?}'",
                    &SignedEntityTypeDiscriminants::CardanoImmutableFilesFull
                )
            })?;
        let mut signed_entities: Vec<SignedEntity<Snapshot>> = Vec::new();

        for record in signed_entities_records {
            signed_entities.push(record.try_into()?);
        }

        Ok(signed_entities)
    }

    async fn get_last_signed_mithril_stake_distributions(
        &self,
        total: usize,
    ) -> StdResult<Vec<SignedEntity<MithrilStakeDistribution>>> {
        let signed_entities_records = self
            .signed_entity_storer
            .get_last_signed_entities_by_type(
                &SignedEntityTypeDiscriminants::MithrilStakeDistribution,
                total,
            )
            .await
            .with_context(|| {
                format!(
                    "Signed Entity Service can not get last signed entities with type: '{:?}'",
                    &SignedEntityTypeDiscriminants::MithrilStakeDistribution
                )
            })?;
        let mut signed_entities: Vec<SignedEntity<MithrilStakeDistribution>> = Vec::new();

        for record in signed_entities_records {
            signed_entities.push(record.try_into()?);
        }

        Ok(signed_entities)
    }

    async fn get_signed_snapshot_by_id(
        &self,
        signed_entity_id: &str,
    ) -> StdResult<Option<SignedEntity<Snapshot>>> {
        let entity: Option<SignedEntity<Snapshot>> = match self
            .signed_entity_storer
            .get_signed_entity(signed_entity_id)
            .await
            .with_context(|| {
                format!(
                    "Signed Entity Service can not get signed entity with id: '{signed_entity_id}'"
                )
            })? {
            Some(entity) => Some(entity.try_into()?),
            None => None,
        };

        Ok(entity)
    }

    async fn get_signed_mithril_stake_distribution_by_id(
        &self,
        signed_entity_id: &str,
    ) -> StdResult<Option<SignedEntity<MithrilStakeDistribution>>> {
        let entity: Option<SignedEntity<MithrilStakeDistribution>> = match self
            .signed_entity_storer
            .get_signed_entity(signed_entity_id)
            .await
            .with_context(|| {
                format!(
                    "Signed Entity Service can not get signed entity with id: '{signed_entity_id}'"
                )
            })? {
            Some(entity) => Some(entity.try_into()?),
            None => None,
        };

        Ok(entity)
    }
}

#[cfg(test)]
mod tests {
    use mithril_common::{entities::Epoch, test_utils::fake_data};

    use super::*;

    use crate::{
        artifact_builder::MockArtifactBuilder, database::provider::MockSignedEntityStorer,
    };

    #[tokio::test]
    async fn build_mithril_stake_distribution_artifact_when_given_mithril_stake_distribution_entity_type(
    ) {
        let signers_with_stake = fake_data::signers_with_stakes(5);
        let mithril_stake_distribution_expected = MithrilStakeDistribution::new(
            Epoch(1),
            signers_with_stake,
            &fake_data::protocol_parameters(),
        );

        let mut mock_signed_entity_storer = MockSignedEntityStorer::new();
        mock_signed_entity_storer
            .expect_store_signed_entity()
            .once()
            .return_once(|_| Ok(()));

        let mut mock_mithril_stake_distribution_artifact_builder =
            MockArtifactBuilder::<Epoch, MithrilStakeDistribution>::new();
        let mithril_stake_distribution_clone = mithril_stake_distribution_expected.clone();
        mock_mithril_stake_distribution_artifact_builder
            .expect_compute_artifact()
            .once()
            .return_once(move |_, _| Ok(mithril_stake_distribution_clone));

        let mithril_stake_distribution_clone = mithril_stake_distribution_expected.clone();
        mock_mithril_stake_distribution_artifact_builder
            .expect_compute_artifact()
            .once()
            .return_once(move |_, _| Ok(mithril_stake_distribution_clone));

        let mock_cardano_immutable_files_full_artifact_builder =
            MockArtifactBuilder::<Beacon, Snapshot>::new();
        let mock_cardano_transactions_artifact_builder =
            MockArtifactBuilder::<Beacon, CardanoTransactionsCommitment>::new();

        let artifact_builder_service = MithrilSignedEntityService::new(
            Arc::new(mock_signed_entity_storer),
            Arc::new(mock_mithril_stake_distribution_artifact_builder),
            Arc::new(mock_cardano_immutable_files_full_artifact_builder),
            Arc::new(mock_cardano_transactions_artifact_builder),
        );
        let certificate = fake_data::certificate("hash".to_string());

        let signed_entity_type = SignedEntityType::MithrilStakeDistribution(Epoch(1));
        let artifact = artifact_builder_service
            .compute_artifact(signed_entity_type.clone(), &certificate)
            .await
            .unwrap();
        let mithril_stake_distribution_computed: MithrilStakeDistribution =
            serde_json::from_str(&serde_json::to_string(&artifact).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_string(&mithril_stake_distribution_expected).unwrap(),
            serde_json::to_string(&mithril_stake_distribution_computed).unwrap()
        );

        artifact_builder_service
            .create_artifact(signed_entity_type, &certificate)
            .await
            .expect("Create artifact should not fail for MithrilStakeDistribution signed entity");
    }

    #[tokio::test]
    async fn build_snapshot_artifact_when_given_cardano_immutable_files_full_entity_type() {
        let snapshot_expected = fake_data::snapshots(1).first().unwrap().to_owned();

        let mut mock_signed_entity_storer = MockSignedEntityStorer::new();
        mock_signed_entity_storer
            .expect_store_signed_entity()
            .once()
            .return_once(|_| Ok(()));

        let mock_mithril_stake_distribution_artifact_builder =
            MockArtifactBuilder::<Epoch, MithrilStakeDistribution>::new();

        let mut mock_cardano_immutable_files_full_artifact_builder =
            MockArtifactBuilder::<Beacon, Snapshot>::new();

        let snapshot_expected_clone = snapshot_expected.clone();
        mock_cardano_immutable_files_full_artifact_builder
            .expect_compute_artifact()
            .times(1)
            .return_once(move |_, _| Ok(snapshot_expected_clone));

        let snapshot_expected_clone = snapshot_expected.clone();
        mock_cardano_immutable_files_full_artifact_builder
            .expect_compute_artifact()
            .times(1)
            .return_once(move |_, _| Ok(snapshot_expected_clone));
        let mock_cardano_transactions_artifact_builder =
            MockArtifactBuilder::<Beacon, CardanoTransactionsCommitment>::new();

        let artifact_builder_service = MithrilSignedEntityService::new(
            Arc::new(mock_signed_entity_storer),
            Arc::new(mock_mithril_stake_distribution_artifact_builder),
            Arc::new(mock_cardano_immutable_files_full_artifact_builder),
            Arc::new(mock_cardano_transactions_artifact_builder),
        );
        let certificate = fake_data::certificate("hash".to_string());

        let signed_entity_type = SignedEntityType::CardanoImmutableFilesFull(Beacon::default());
        let artifact = artifact_builder_service
            .compute_artifact(signed_entity_type.clone(), &certificate)
            .await
            .unwrap();
        let snapshot_computed: Snapshot =
            serde_json::from_str(&serde_json::to_string(&artifact).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_string(&snapshot_expected).unwrap(),
            serde_json::to_string(&snapshot_computed).unwrap()
        );

        artifact_builder_service
            .create_artifact(signed_entity_type, &certificate)
            .await
            .expect("Create artifact should not fail for CardanoImmutableFilesFull signed entity");
    }

    #[tokio::test]
    async fn build_artifact_for_cardano_transactions_store_nothing_in_db() {
        let mut mock_signed_entity_storer = MockSignedEntityStorer::new();
        mock_signed_entity_storer
            .expect_store_signed_entity()
            .never();

        let mock_mithril_stake_distribution_artifact_builder =
            MockArtifactBuilder::<Epoch, MithrilStakeDistribution>::new();
        let mock_cardano_immutable_files_full_artifact_builder =
            MockArtifactBuilder::<Beacon, Snapshot>::new();
        let mock_cardano_transactions_artifact_builder =
            MockArtifactBuilder::<Beacon, CardanoTransactionsCommitment>::new();

        let artifact_builder_service = MithrilSignedEntityService::new(
            Arc::new(mock_signed_entity_storer),
            Arc::new(mock_mithril_stake_distribution_artifact_builder),
            Arc::new(mock_cardano_immutable_files_full_artifact_builder),
            Arc::new(mock_cardano_transactions_artifact_builder),
        );

        let certificate = fake_data::certificate("hash".to_string());

        let signed_entity_type = SignedEntityType::CardanoTransactions(Beacon::default());
        let artifact = artifact_builder_service
            .compute_artifact(signed_entity_type.clone(), &certificate)
            .await
            .unwrap();

        assert!(artifact.is_none());

        artifact_builder_service
            .create_artifact(signed_entity_type, &certificate)
            .await
            .expect("Create artifact should not fail for CardanoTransactions signed entity");
    }
}
