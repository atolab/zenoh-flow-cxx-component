use cxx::UniquePtr;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use zenoh_flow::{
    downcast_mut, runtime::message::SerDeData, Component, ComponentOutput, Data, DowncastAny,
    InputRule, Operator, OutputRule, State, Token, TokenAction, ZFError, ZFResult,
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
        include!("zenoh-flow-cxx-operator/cpp/include/operator.hpp");

        type State;

        fn initialize(configuration: &ConfigurationMap) -> UniquePtr<State>;

        fn input_rule(
            context: &mut Context,
            state: &mut UniquePtr<State>,
            tokens: &mut Vec<Token>,
        ) -> Result<bool>;

        fn run(
            context: &mut Context,
            state: &mut UniquePtr<State>,
            inputs: Vec<Input>,
        ) -> Result<Vec<Output>>;
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

impl ffi::Token {
    pub fn from_token(token: &Token, port_id: &str) -> ZFResult<Self> {
        match token {
            Token::NotReady => Ok(Self {
                status: ffi::TokenStatus::Pending,
                action: ffi::TokenAction::Wait,
                port_id: port_id.to_string(),
                data: Vec::new(),
                timestamp: 0,
            }),

            Token::Ready(token) => {
                let data = match &token.data.data {
                    SerDeData::Serialized(ser) => ser.as_ref().clone(),
                    SerDeData::Deserialized(de) => de.try_serialize()?,
                };

                Ok(Self {
                    status: ffi::TokenStatus::Ready,
                    action: ffi::TokenAction::Consume,
                    port_id: port_id.to_string(),
                    data,
                    timestamp: token.data.timestamp.get_time().as_u64(),
                })
            }
        }
    }
}

impl From<&mut zenoh_flow::Context> for ffi::Context {
    fn from(context: &mut zenoh_flow::Context) -> Self {
        Self { mode: context.mode }
    }
}

impl From<TokenAction> for ffi::TokenAction {
    fn from(action: TokenAction) -> Self {
        match action {
            TokenAction::Consume => ffi::TokenAction::Consume,
            TokenAction::Drop => ffi::TokenAction::Drop,
            TokenAction::KeepRun => ffi::TokenAction::Keep,
            TokenAction::Keep => ffi::TokenAction::Keep,
            TokenAction::Wait => ffi::TokenAction::Wait,
        }
    }
}

impl ffi::Input {
    fn from_data_message(
        port_id: &str,
        data_message: &zenoh_flow::runtime::message::DataMessage,
    ) -> ZFResult<Self> {
        let data = match &data_message.data {
            SerDeData::Serialized(ser) => ser.as_ref().clone(),
            SerDeData::Deserialized(de) => de.try_serialize()?,
        };

        Ok(Self {
            port_id: port_id.to_string(),
            data,
            timestamp: data_message.timestamp.get_time().as_u64(),
        })
    }
}

/*

Operator implementation.

*/
pub struct MyOperator;

impl Component for MyOperator {
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

    fn clean(&self, _state: &mut Box<dyn zenoh_flow::State>) -> ZFResult<()> {
        Ok(())
    }
}

impl InputRule for MyOperator {
    fn input_rule(
        &self,
        context: &mut zenoh_flow::Context,
        dyn_state: &mut Box<dyn zenoh_flow::State>,
        tokens: &mut HashMap<zenoh_flow::PortId, zenoh_flow::Token>,
    ) -> zenoh_flow::ZFResult<bool> {
        let wrapper = downcast_mut!(StateWrapper, dyn_state).unwrap();
        let res_cxx_tokens: Result<Vec<ffi::Token>, ZFError> = tokens
            .iter()
            .map(|(port_id, token)| ffi::Token::from_token(token, port_id))
            .collect();
        let mut cxx_tokens = res_cxx_tokens?;
        let mut cxx_context = ffi::Context::from(context);

        {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::input_rule(&mut cxx_context, &mut wrapper.state, &mut cxx_tokens)
                    .map_err(|_| ZFError::GenericError)
            }
        }
    }
}

impl OutputRule for MyOperator {
    fn output_rule(
        &self,
        _context: &mut zenoh_flow::Context,
        _dyn_state: &mut Box<dyn zenoh_flow::State>,
        outputs: &HashMap<zenoh_flow::PortId, std::sync::Arc<dyn zenoh_flow::Data>>,
    ) -> ZFResult<HashMap<zenoh_flow::PortId, zenoh_flow::ComponentOutput>> {
        let mut results = HashMap::with_capacity(outputs.len());
        // NOTE: default output rule for now.
        for (port_id, data) in outputs {
            results.insert(port_id.clone(), ComponentOutput::Data(data.clone()));
        }

        Ok(results)
    }
}

impl Operator for MyOperator {
    fn run(
        &self,
        context: &mut zenoh_flow::Context,
        dyn_state: &mut Box<dyn zenoh_flow::State>,
        inputs: &mut HashMap<zenoh_flow::PortId, zenoh_flow::runtime::message::DataMessage>,
    ) -> ZFResult<HashMap<zenoh_flow::PortId, std::sync::Arc<dyn zenoh_flow::Data>>> {
        let mut cxx_context = ffi::Context::from(context);
        let wrapper = downcast_mut!(StateWrapper, dyn_state).unwrap();
        let result_cxx_inputs: Result<Vec<ffi::Input>, ZFError> = inputs
            .iter()
            .map(|(port_id, data_message)| ffi::Input::from_data_message(port_id, data_message))
            .collect();
        let cxx_inputs = result_cxx_inputs?;

        let cxx_outputs = {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::run(&mut cxx_context, &mut wrapper.state, cxx_inputs)
                    .map_err(|_| ZFError::GenericError)?
            }
        };

        let mut result: HashMap<zenoh_flow::PortId, Arc<dyn zenoh_flow::Data>> =
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

zenoh_flow::export_operator!(register);

fn register() -> ZFResult<Arc<dyn Operator>> {
    Ok(Arc::new(MyOperator) as Arc<dyn Operator>)
}
