#include "zenoh-flow-cxx-sink/cpp/include/sink.hpp"
#include "zenoh-flow-cxx-sink/src/lib.rs.h"
#include <cstdint>
#include <cstring>
#include <memory>
#include <string>
#include <vector>
#include <iostream>

namespace zenoh {
namespace flow {

State::State() {}

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

void
run(Context &context, std::unique_ptr<State> &state, rust::Vec<Input> inputs) {
  for (auto input : inputs) {
    std::cout << "Received on <" << input.port_id << ">: " << std::endl;
    std::cout << "\t";
    for (unsigned char c: input.data) {
      std::cout << unsigned(c);
    }
    std::cout << std::endl << std::flush;
  }
}

} // namespace flow
} // namespace zenoh
