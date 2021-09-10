#include "zenoh-flow-cxx-operator/cpp/include/operator.hpp"
#include "zenoh-flow-cxx-operator/src/lib.rs.h"
#include <cstdint>
#include <cstring>
#include <memory>
#include <string>
#include <vector>

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

template< typename T >
T& from_bytes( const rust::Vec<byte_t>& bytes, T& object )
{
    // http://en.cppreference.com/w/cpp/types/is_trivially_copyable
    static_assert( std::is_trivially_copyable<T>::value, "not a TriviallyCopyable type" ) ;

    byte_t* begin_object = reinterpret_cast< byte_t* >( std::addressof(object) ) ;
    std::copy( bytes.begin(), bytes.end(), begin_object ) ;

    return object ;
}

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
  std::cout << "[Fizz] Input rule says: ";

  for (auto token : tokens) {
    if (token.status != TokenStatus::Ready) {
      std::cout << "no" << std::endl;
        return false;
      }
  }

  std::cout << "ok" << std::endl;
  return true;
}

rust::Vec<Output>
run(Context &context, std::unique_ptr<State> &state, rust::Vec<Input> inputs) {
  std::uint64_t number = 0;
  from_bytes(inputs[0].data, number);

  std::string fizz_str = "";
  if (number % 2 == 0) {
    fizz_str = "(C++) fizz";
  }
  fizz_str.push_back('\0');

  Output fizz_output { "fizz", to_bytes(fizz_str) };
  std::vector<Output> results { fizz_output };
  return to_rust_vec(results);
}
} // namespace flow
} // namespace zenoh

