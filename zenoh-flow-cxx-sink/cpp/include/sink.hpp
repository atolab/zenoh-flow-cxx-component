#pragma once
#include <algorithm>
#include <memory>
#include <vector>
#include "rust/cxx.h"
#include <iostream>
#include "zenoh-flow-cxx-sink/../include/zenoh_flow.hpp"

namespace zenoh {
namespace flow {

class State {
public:
  State ();
};

std::unique_ptr<State> initialize(const ConfigurationMap &configuration);
bool input_rule(Context &context, std::unique_ptr<State> &state,
                rust::Vec<Token> &tokens);
void run(Context &context, std::unique_ptr<State> &state,
         rust::Vec<Input> inputs);

} // namespace flow
} // namespace zenoh
