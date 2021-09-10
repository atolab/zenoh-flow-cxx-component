#include "zenoh-flow-cxx-source/cpp/include/zenoh_flow.hpp"
#include "zenoh-flow-cxx-source/src/lib.rs.h"
#include <cstdint>
#include <cstring>
#include <memory>
#include <string>
#include <vector>
#include <sstream>

namespace zenoh_flow {

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

std::unique_ptr<ZFCxxState>
initialize(const ZFCxxConfigurationMap &configuration)
{
  //
  // /!\ NOTE: `make_unique` requires "c++14"
  //
  return std::make_unique<ZFCxxState>();
}

rust::Vec<ZFCxxOutput>
run(ZFCxxContext &context, std::unique_ptr<ZFCxxState> &state)
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

  ZFCxxOutput manual_source_output { "number", to_bytes(number) };

  std::vector<ZFCxxOutput> results(1);
  results.push_back(manual_source_output);

  // std::cout << "[source] Returning… "<< std::endl;
  return to_rust_vec<ZFCxxOutput>(results);
}
} // namespace zenoh_flow
