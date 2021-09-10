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

template < typename T >
rust::Vec<T> to_rust_vec(const std::vector<T>& v)
{
  rust::Vec<T> res {};
  res.reserve(v.size());
  for (auto item : v) {
    res.push_back(item);
  }

  return res;
}

template <typename T>
rust::Vec<byte_t> to_bytes( const T& object ) {
  std::vector< byte_t > bytes(sizeof(T)) ;

  const byte_t *begin = reinterpret_cast<const byte_t *>(std::addressof(object));
  const byte_t *end = begin + sizeof(T);
  // std::copy(begin, end, &bytes[0]);

  return to_rust_vec<byte_t>(std::vector<byte_t>(begin, end));
}

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
  std::uint64_t number;
  std::string input;

  std::cout << "[WAITING FOR USER INPUT BEFORE LAUNCHING]" << std::endl;
  std::getline(std::cin, input);
  input.clear();

  std::cout << "Enter a number: ";
  std::getline(std::cin, input);
  std::stringstream(input) >> number;
  std::cout << std::endl;

  Output manual_source_output { "number", to_bytes(number) };

  std::vector<Output> results(1);
  results.push_back(manual_source_output);

  return to_rust_vec<Output>(results);
}
} // namespace flow
} // namespace zenoh
