use async_trait::async_trait;
use cxx::UniquePtr;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use zenoh_flow::{
    downcast_mut, ZFComponent, ZFComponentOutput, ZFComponentOutputRule, ZFContext, ZFDataTrait,
    ZFDowncastAny, ZFError, ZFPortID, ZFResult, ZFSourceTrait, ZFStateTrait,
};

extern crate zenoh_flow;

#[cxx::bridge(namespace = "zenoh_flow")]
pub mod ffi {
    // Shared structures between Rust and C++
    pub struct ZFCxxContext {
        pub mode: usize,
    }

    pub enum ZFCxxTokenStatus {
        Pending,
        Ready,
        DeadlineMiss,
    }

    pub enum ZFCxxTokenAction {
        Consume,
        Drop,
        Keep,
        Postpone,
        Wait,
    }

    pub struct ZFCxxToken {
        pub status: ZFCxxTokenStatus,
        pub action: ZFCxxTokenAction,
        pub port_id: String,
        pub data: Vec<u8>,
        pub timestamp: u64,
    }

    pub struct ZFCxxInput {
        pub port_id: String,
        pub data: Vec<u8>,
        pub timestamp: u64,
    }

    pub struct ZFCxxOutput {
        pub port_id: String,
        pub data: Vec<u8>,
    }

    pub struct ZFCxxData {
        pub bytes: Vec<u8>,
    }

    pub struct ZFCxxConfiguration {
        pub key: String,
        pub value: String,
    }

    pub struct ZFCxxConfigurationMap {
        pub map: Vec<ZFCxxConfiguration>,
    }

    unsafe extern "C++" {
        include!("zenoh-flow-cxx-source/cpp/include/zenoh_flow.hpp");

        type ZFCxxState;

        fn initialize(configuration: &ZFCxxConfigurationMap) -> UniquePtr<ZFCxxState>;

        fn run(
            context: &mut ZFCxxContext,
            state: &mut UniquePtr<ZFCxxState>,
        ) -> Result<Vec<ZFCxxOutput>>;
    }
}

impl From<HashMap<String, String>> for ffi::ZFCxxConfigurationMap {
    fn from(configuration: HashMap<String, String>) -> Self {
        ffi::ZFCxxConfigurationMap {
            map: configuration
                .iter()
                .map(|(key, value)| ffi::ZFCxxConfiguration {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect(),
        }
    }
}

unsafe impl Send for ffi::ZFCxxState {}
unsafe impl Sync for ffi::ZFCxxState {}

pub struct ZFCxxStateWrapper {
    pub state: UniquePtr<ffi::ZFCxxState>,
}

impl ZFStateTrait for ZFCxxStateWrapper {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Debug for ZFCxxStateWrapper {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ffi::ZFCxxData {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl Debug for ffi::ZFCxxData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZFCxxData")
            .field("bytes", &self.bytes)
            .finish()
    }
}

impl ZFDowncastAny for ffi::ZFCxxData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ZFDataTrait for ffi::ZFCxxData {
    fn try_serialize(&self) -> ZFResult<Vec<u8>> {
        Ok(self.bytes.clone())
    }
}

impl From<&mut ZFContext> for ffi::ZFCxxContext {
    fn from(context: &mut ZFContext) -> Self {
        Self { mode: context.mode }
    }
}

/*

ZFCxxSource implementation.

*/
pub struct ZFCxxSource;

impl ZFComponent for ZFCxxSource {
    fn initialize(
        &self,
        configuration: &Option<std::collections::HashMap<String, String>>,
    ) -> Box<dyn zenoh_flow::ZFStateTrait> {
        let configuration = match configuration {
            Some(config) => ffi::ZFCxxConfigurationMap::from(config.clone()),
            None => ffi::ZFCxxConfigurationMap { map: Vec::new() },
        };

        let state = {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::initialize(&configuration)
            }
        };
        Box::new(ZFCxxStateWrapper { state })
    }

    fn clean(&self, _state: &mut Box<dyn ZFStateTrait>) -> ZFResult<()> {
        Ok(())
    }
}

impl ZFComponentOutputRule for ZFCxxSource {
    fn output_rule(
        &self,
        _context: &mut ZFContext,
        _dyn_state: &mut Box<dyn ZFStateTrait>,
        outputs: &HashMap<String, std::sync::Arc<dyn zenoh_flow::ZFDataTrait>>,
    ) -> ZFResult<HashMap<zenoh_flow::ZFPortID, zenoh_flow::ZFComponentOutput>> {
        let mut results = HashMap::with_capacity(outputs.len());
        // NOTE: default output rule for now.
        for (port_id, data) in outputs {
            results.insert(port_id.to_string(), ZFComponentOutput::Data(data.clone()));
        }

        Ok(results)
    }
}

#[async_trait]
impl ZFSourceTrait for ZFCxxSource {
    async fn run(
        &self,
        context: &mut ZFContext,
        dyn_state: &mut Box<dyn ZFStateTrait>,
    ) -> ZFResult<HashMap<ZFPortID, Arc<dyn ZFDataTrait>>> {
        let mut cxx_context = ffi::ZFCxxContext::from(context);
        let wrapper = downcast_mut!(ZFCxxStateWrapper, dyn_state).unwrap();

        let cxx_outputs = {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::run(&mut cxx_context, &mut wrapper.state).map_err(|_| ZFError::GenericError)?
            }
        };

        let mut result: HashMap<ZFPortID, Arc<dyn ZFDataTrait>> =
            HashMap::with_capacity(cxx_outputs.len());
        for cxx_output in cxx_outputs.into_iter() {
            result.insert(
                cxx_output.port_id.to_owned(),
                Arc::new(ffi::ZFCxxData::new(cxx_output.data)),
            );
        }

        Ok(result)
    }
}

zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn ZFSourceTrait>> {
    Ok(Arc::new(ZFCxxSource) as Arc<dyn ZFSourceTrait>)
}
