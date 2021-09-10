#pragma once
#include <algorithm>
#include <memory>
#include <vector>
#include "rust/cxx.h"
#include "zenoh-flow-cxx-source/../include/zenoh_flow.hpp"

namespace zenoh {
namespace flow {

class State {
public:
  State();
};

std::unique_ptr<State>
initialize(const ConfigurationMap &configuration);
rust::Vec<Output> run(Context &context, std::unique_ptr<State> &state);

} // namespace flow
} // namespace zenoh
