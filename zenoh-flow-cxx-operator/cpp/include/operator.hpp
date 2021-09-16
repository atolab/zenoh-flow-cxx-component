#pragma once
#include <algorithm>
#include <cstdint>
#include <memory>
#include <vector>
#include "rust/cxx.h"
#include <iostream>
#include "zenoh-flow-cxx-operator/../include/zenoh_flow.hpp"

namespace zenoh {
namespace flow {

class State {
private:
  std::uint8_t counter;
public:
  State ();
  void increaseCounter ();
  std::uint8_t getCounter ();
};

std::unique_ptr<State> initialize(const ConfigurationMap &configuration);
bool input_rule(Context &context, std::unique_ptr<State> &state,
                rust::Vec<Token> &tokens);
rust::Vec<Output> run(Context &context,
                      std::unique_ptr<State> &state,
                      rust::Vec<Input> inputs);

} // namespace flow
} // namespace zenoh
