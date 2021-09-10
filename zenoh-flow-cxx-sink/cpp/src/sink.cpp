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

using byte_t = unsigned char ;

template< typename T >
T& from_bytes( const rust::Vec<byte_t>& bytes, T& object )
{
    // http://en.cppreference.com/w/cpp/types/is_trivially_copyable
    // static_assert( std::is_trivially_copyable<T>::value, "not a TriviallyCopyable type" ) ;

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
    std::cout << "Received on <" << input.port_id << ">: ";
    if (input.port_id == "fizz") {
      std::string fizz = "";
      from_bytes(input.data, fizz);
      std::cout << fizz << std::endl;
    }
  }
}

} // namespace flow
} // namespace zenoh
