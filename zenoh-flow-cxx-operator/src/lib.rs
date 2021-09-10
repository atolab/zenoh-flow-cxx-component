use cxx::UniquePtr;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use zenoh_flow::{
    downcast_mut,
    runtime::message::{ZFDataMessage, ZFSerDeData},
    Token, TokenAction, ZFComponent, ZFComponentInputRule, ZFComponentOutput,
    ZFComponentOutputRule, ZFContext, ZFDataTrait, ZFDowncastAny, ZFError, ZFOperatorTrait,
    ZFPortID, ZFResult, ZFStateTrait,
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
        include!("zenoh-flow-cxx-operator/cpp/include/zenoh_flow.hpp");

        type ZFCxxState;

        fn initialize(configuration: &ZFCxxConfigurationMap) -> UniquePtr<ZFCxxState>;

        fn input_rule(
            context: &mut ZFCxxContext,
            state: &mut UniquePtr<ZFCxxState>,
            tokens: &mut Vec<ZFCxxToken>,
        ) -> Result<bool>;

        fn run(
            context: &mut ZFCxxContext,
            state: &mut UniquePtr<ZFCxxState>,
            inputs: Vec<ZFCxxInput>,
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

impl ffi::ZFCxxToken {
    pub fn from_token(token: &Token, port_id: &str) -> ZFResult<Self> {
        match token {
            Token::NotReady => Ok(Self {
                status: ffi::ZFCxxTokenStatus::Pending,
                action: ffi::ZFCxxTokenAction::Wait,
                port_id: port_id.to_string(),
                data: Vec::new(),
                timestamp: 0,
            }),

            Token::Ready(token) => {
                let data = match &token.data.data {
                    ZFSerDeData::Serialized(ser) => ser.as_ref().clone(),
                    ZFSerDeData::Deserialized(de) => de.try_serialize()?,
                };

                Ok(Self {
                    status: ffi::ZFCxxTokenStatus::Ready,
                    action: ffi::ZFCxxTokenAction::Consume,
                    port_id: port_id.to_string(),
                    data,
                    timestamp: token.data.timestamp.get_time().as_u64(),
                })
            }
        }
    }
}

impl From<&mut ZFContext> for ffi::ZFCxxContext {
    fn from(context: &mut ZFContext) -> Self {
        Self { mode: context.mode }
    }
}

impl From<TokenAction> for ffi::ZFCxxTokenAction {
    fn from(action: TokenAction) -> Self {
        match action {
            TokenAction::Consume => ffi::ZFCxxTokenAction::Consume,
            TokenAction::Drop => ffi::ZFCxxTokenAction::Drop,
            TokenAction::KeepRun => ffi::ZFCxxTokenAction::Keep,
            TokenAction::Keep => ffi::ZFCxxTokenAction::Keep,
            TokenAction::Wait => ffi::ZFCxxTokenAction::Wait,
        }
    }
}

impl ffi::ZFCxxInput {
    fn from_data_message(port_id: &str, data_message: &ZFDataMessage) -> ZFResult<Self> {
        let data = match &data_message.data {
            ZFSerDeData::Serialized(ser) => ser.as_ref().clone(),
            ZFSerDeData::Deserialized(de) => de.try_serialize()?,
        };

        Ok(Self {
            port_id: port_id.to_string(),
            data,
            timestamp: data_message.timestamp.get_time().as_u64(),
        })
    }
}

/*

ZFCxxOperator implementation.

*/
pub struct ZFCxxOperator;

impl ZFComponent for ZFCxxOperator {
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

impl ZFComponentInputRule for ZFCxxOperator {
    fn input_rule(
        &self,
        context: &mut zenoh_flow::ZFContext,
        dyn_state: &mut Box<dyn ZFStateTrait>,
        tokens: &mut HashMap<String, zenoh_flow::Token>,
    ) -> zenoh_flow::ZFResult<bool> {
        let wrapper = downcast_mut!(ZFCxxStateWrapper, dyn_state).unwrap();
        let res_cxx_tokens: Result<Vec<ffi::ZFCxxToken>, ZFError> = tokens
            .iter()
            .map(|(port_id, token)| ffi::ZFCxxToken::from_token(token, port_id))
            .collect();
        let mut cxx_tokens = res_cxx_tokens?;
        let mut cxx_context = ffi::ZFCxxContext::from(context);

        {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::input_rule(&mut cxx_context, &mut wrapper.state, &mut cxx_tokens)
                    .map_err(|_| ZFError::GenericError)
            }
        }
    }
}

impl ZFComponentOutputRule for ZFCxxOperator {
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

impl ZFOperatorTrait for ZFCxxOperator {
    fn run(
        &self,
        context: &mut ZFContext,
        dyn_state: &mut Box<dyn ZFStateTrait>,
        inputs: &mut HashMap<String, ZFDataMessage>,
    ) -> ZFResult<HashMap<zenoh_flow::ZFPortID, std::sync::Arc<dyn zenoh_flow::ZFDataTrait>>> {
        let mut cxx_context = ffi::ZFCxxContext::from(context);
        let wrapper = downcast_mut!(ZFCxxStateWrapper, dyn_state).unwrap();
        let result_cxx_inputs: Result<Vec<ffi::ZFCxxInput>, ZFError> = inputs
            .iter()
            .map(|(port_id, data_message)| {
                ffi::ZFCxxInput::from_data_message(port_id, data_message)
            })
            .collect();
        let cxx_inputs = result_cxx_inputs?;

        let cxx_outputs = {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::run(&mut cxx_context, &mut wrapper.state, cxx_inputs)
                    .map_err(|_| ZFError::GenericError)?
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

zenoh_flow::export_operator!(register);

fn register() -> ZFResult<Arc<dyn ZFOperatorTrait>> {
    Ok(Arc::new(ZFCxxOperator) as Arc<dyn ZFOperatorTrait>)
}
