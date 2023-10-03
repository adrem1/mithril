use crate::{
    dependency_injection::MultiSignerWrapper,
    event_store::{EventMessage, TransmitterService},
    services::CertifierService,
    services::{SignedEntityService, TickerService},
    CertificatePendingStore, Configuration, DependencyContainer, ProtocolParametersStorer,
    SignerRegisterer, VerificationKeyStorer,
};

use crate::database::provider::SignerGetter;
use mithril_common::{api_version::APIVersionProvider, BeaconProvider};
use std::convert::Infallible;
use std::sync::Arc;
use warp::Filter;

/// With certificate pending store
pub(crate) fn with_certificate_pending_store(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<CertificatePendingStore>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.certificate_pending_store.clone())
}

/// With protocol parameters store
pub(crate) fn with_protocol_parameters_store(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<dyn ProtocolParametersStorer>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.protocol_parameters_store.clone())
}

/// With multi signer middleware
pub fn with_multi_signer(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (MultiSignerWrapper,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.multi_signer.clone())
}

/// With signer registerer middleware
pub fn with_signer_registerer(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<dyn SignerRegisterer>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.signer_registerer.clone())
}

/// With signer getter middleware
pub fn with_signer_getter(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<dyn SignerGetter>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.signer_getter.clone())
}

/// With config middleware
pub fn with_config(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Configuration,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.config.clone())
}

/// With Event transmitter middleware
pub fn with_event_transmitter(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<TransmitterService<EventMessage>>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.event_transmitter.clone())
}

/// With round_opener middleware
pub fn with_beacon_provider(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<dyn BeaconProvider>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.beacon_provider.clone())
}

/// With certifier service middleware
pub fn with_certifier_service(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<dyn CertifierService>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.certifier_service.clone())
}

/// With ticker service middleware
pub fn with_ticker_service(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<dyn TickerService>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.ticker_service.clone())
}

/// With signed entity service
pub fn with_signed_entity_service(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<dyn SignedEntityService>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.signed_entity_service.clone())
}

/// With verification key store
pub fn with_verification_key_store(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<dyn VerificationKeyStorer>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.verification_key_store.clone())
}

/// With API version provider
pub fn with_api_version_provider(
    dependency_manager: Arc<DependencyContainer>,
) -> impl Filter<Extract = (Arc<APIVersionProvider>,), Error = Infallible> + Clone {
    warp::any().map(move || dependency_manager.api_version_provider.clone())
}
