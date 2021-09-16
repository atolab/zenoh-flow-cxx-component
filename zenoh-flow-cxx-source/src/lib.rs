use async_trait::async_trait;
use cxx::UniquePtr;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use zenoh_flow::{
    downcast_mut, Component, ComponentOutput, Context, Data, DowncastAny, OutputRule, PortId,
    Source, State, ZFError, ZFResult,
};

extern crate zenoh_flow;

#[cxx::bridge(namespace = "zenoh::flow")]
pub mod ffi {
    // Shared structures between Rust and C++
    pub struct Context {
        pub mode: usize,
    }

    pub enum TokenStatus {
        Pending,
        Ready,
        DeadlineMiss,
    }

    pub enum TokenAction {
        Consume,
        Drop,
        Keep,
        Postpone,
        Wait,
    }

    pub struct Token {
        pub status: TokenStatus,
        pub action: TokenAction,
        pub port_id: String,
        pub data: Vec<u8>,
        pub timestamp: u64,
    }

    pub struct Input {
        pub port_id: String,
        pub data: Vec<u8>,
        pub timestamp: u64,
    }

    pub struct Output {
        pub port_id: String,
        pub data: Vec<u8>,
    }

    pub struct Data {
        pub bytes: Vec<u8>,
    }

    pub struct Configuration {
        pub key: String,
        pub value: String,
    }

    pub struct ConfigurationMap {
        pub map: Vec<Configuration>,
    }

    unsafe extern "C++" {
        include!("zenoh-flow-cxx-source/cpp/include/source.hpp");

        type State;

        fn initialize(configuration: &ConfigurationMap) -> UniquePtr<State>;

        fn run(context: &mut Context, state: &mut UniquePtr<State>) -> Result<Vec<Output>>;
    }
}

impl From<HashMap<String, String>> for ffi::ConfigurationMap {
    fn from(configuration: HashMap<String, String>) -> Self {
        ffi::ConfigurationMap {
            map: configuration
                .iter()
                .map(|(key, value)| ffi::Configuration {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect(),
        }
    }
}

unsafe impl Send for ffi::State {}
unsafe impl Sync for ffi::State {}

pub struct StateWrapper {
    pub state: UniquePtr<ffi::State>,
}

impl State for StateWrapper {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Debug for StateWrapper {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ffi::Data {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl Debug for ffi::Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data").field("bytes", &self.bytes).finish()
    }
}

impl DowncastAny for ffi::Data {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Data for ffi::Data {
    fn try_serialize(&self) -> ZFResult<Vec<u8>> {
        Ok(self.bytes.clone())
    }
}

impl From<&mut Context> for ffi::Context {
    fn from(context: &mut Context) -> Self {
        Self { mode: context.mode }
    }
}

/*

Source implementation.

*/
pub struct MySource;

impl Component for MySource {
    fn initialize(
        &self,
        configuration: &Option<std::collections::HashMap<String, String>>,
    ) -> Box<dyn zenoh_flow::State> {
        let configuration = match configuration {
            Some(config) => ffi::ConfigurationMap::from(config.clone()),
            None => ffi::ConfigurationMap { map: Vec::new() },
        };

        let state = {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::initialize(&configuration)
            }
        };
        Box::new(StateWrapper { state })
    }

    fn clean(&self, _state: &mut Box<dyn State>) -> ZFResult<()> {
        Ok(())
    }
}

impl OutputRule for MySource {
    fn output_rule(
        &self,
        _context: &mut Context,
        _dyn_state: &mut Box<dyn State>,
        outputs: &HashMap<PortId, std::sync::Arc<dyn zenoh_flow::Data>>,
    ) -> ZFResult<HashMap<zenoh_flow::PortId, zenoh_flow::ComponentOutput>> {
        let mut results = HashMap::with_capacity(outputs.len());
        // NOTE: default output rule for now.
        for (port_id, data) in outputs {
            results.insert(port_id.clone(), ComponentOutput::Data(data.clone()));
        }

        Ok(results)
    }
}

#[async_trait]
impl Source for MySource {
    async fn run(
        &self,
        context: &mut Context,
        dyn_state: &mut Box<dyn zenoh_flow::State>,
    ) -> ZFResult<HashMap<PortId, Arc<dyn Data>>> {
        let mut cxx_context = ffi::Context::from(context);
        let wrapper = downcast_mut!(StateWrapper, dyn_state).unwrap();

        let cxx_outputs = {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::run(&mut cxx_context, &mut wrapper.state).map_err(|_| ZFError::GenericError)?
            }
        };

        let mut result: HashMap<PortId, Arc<dyn zenoh_flow::Data>> =
            HashMap::with_capacity(cxx_outputs.len());
        for cxx_output in cxx_outputs.into_iter() {
            result.insert(
                cxx_output.port_id.into(),
                Arc::new(ffi::Data::new(cxx_output.data)),
            );
        }

        Ok(result)
    }
}

zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(MySource) as Arc<dyn Source>)
}
