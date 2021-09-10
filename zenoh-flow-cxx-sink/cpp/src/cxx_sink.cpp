#include "zenoh-flow-cxx-sink/cpp/include/zenoh_flow.hpp"
#include "zenoh-flow-cxx-sink/src/lib.rs.h"
#include <cstdint>
#include <cstring>
#include <memory>
#include <string>
#include <vector>
#include <iostream>

namespace zenoh_flow {

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

std::unique_ptr<ZFCxxState>
initialize(const ZFCxxConfigurationMap &configuration) {
  //
  // /!\ NOTE: `make_unique` requires "c++14"
  //
  return std::make_unique<ZFCxxState>();
}

bool
input_rule(ZFCxxContext &context, std::unique_ptr<ZFCxxState> &state, rust::Vec<ZFCxxToken> &tokens) {
  for (auto token : tokens) {
    if (token.status != ZFCxxTokenStatus::Ready) {
        return false;
      }
  }

  return true;
}

void
run(ZFCxxContext &context, std::unique_ptr<ZFCxxState> &state, rust::Vec<ZFCxxInput> inputs) {
  for (auto input : inputs) {
    std::cout << "Received on <" << input.port_id << ">: ";
    if (input.port_id == "fizz") {
      std::string fizz = "";
      from_bytes(input.data, fizz);
      std::cout << fizz << std::endl;
    }
  }
}
}

