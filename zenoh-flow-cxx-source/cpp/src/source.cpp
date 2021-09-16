#include "zenoh-flow-cxx-source/cpp/include/source.hpp"
#include "zenoh-flow-cxx-source/src/lib.rs.h"
#include <cstdint>
#include <cstring>
#include <memory>
#include <string>
#include <vector>
#include <sstream>

namespace zenoh {
namespace flow {

using byte_t = unsigned char ;

State::State() {}

std::unique_ptr<State>
initialize(const ConfigurationMap &configuration)
{
  //
  // /!\ NOTE: `make_unique` requires "c++14"
  //
  return std::make_unique<State>();
}

rust::Vec<Output>
run(Context &context, std::unique_ptr<State> &state)
{
  std::string input;

  std::cout << "Press ENTER.";
  std::getline(std::cin, input);
  std::cout << std::endl;

  rust::Vec<byte_t> tick = { 1 };

  Output output { "tick", tick };

  rust::Vec<Output> results { output };
  return results;
}
} // namespace flow
} // namespace zenoh
