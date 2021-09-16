#include "zenoh-flow-cxx-operator/cpp/include/operator.hpp"
#include "zenoh-flow-cxx-operator/src/lib.rs.h"
#include <algorithm>
#include <cstdint>
#include <cstring>
#include <memory>
#include <string>
#include <vector>

namespace zenoh {
namespace flow {

State::State() {
  counter = 0;
}

void State::increaseCounter(void) {
  counter += 1;
}

std::uint8_t State::getCounter(void) {
  return counter;
}

std::unique_ptr<State>
initialize(const ConfigurationMap &configuration) {
  //
  // /!\ NOTE: `make_unique` requires "c++14"
  //
  return std::make_unique<State>();
}

bool
input_rule(Context &context, std::unique_ptr<State> &state, rust::Vec<Token> &tokens) {
  for (auto token : tokens) {
    if (token.status != TokenStatus::Ready) {
        return false;
      }
  }

  return true;
}

rust::Vec<Output>
run(Context &context, std::unique_ptr<State> &state, rust::Vec<Input> inputs) {
  state->increaseCounter();
  rust::Vec<std::uint8_t> counter = { state->getCounter() };
  Output count { "count", counter };
  rust::Vec<Output> results { count };
  return results;
}
} // namespace flow
} // namespace zenoh

