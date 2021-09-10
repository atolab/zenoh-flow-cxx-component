#pragma once
#include <algorithm>
#include <memory>
#include <vector>
#include "rust/cxx.h"
#include <iostream>
#include <string>

namespace zenoh_flow {

struct ZFCxxContext;
struct ZFCxxToken;
struct ZFCxxInput;
struct ZFCxxOutput;
// struct ZFCxxOutput {
//   std::string port_id;
//   std::unique_ptr< std::vector<unsigned char> > data;

//   ZFCxxOutput(std::string port_id,
//               std::unique_ptr< std::vector<unsigned char> > data);
// };
struct ZFCxxData;
struct ZFCxxConfiguration;
struct ZFCxxConfigurationMap;

class ZFCxxState {
private:
  std::vector<int> previous_numbers;

public:
  ZFCxxState () {}
  void AddNumber(int number) { previous_numbers.push_back(number); }
  void DisplayPreviousNumbers() {
    if (previous_numbers.size() > 0) {
      std::cout << "[ZFCxxState] " << previous_numbers[0];
    }

    for (unsigned int i = 1; i < previous_numbers.size(); i++) {
      std::cout << ", " << previous_numbers[i];
    }

    std::cout << std::endl;
  }
};

  std::unique_ptr<ZFCxxState> initialize(const ZFCxxConfigurationMap &configuration);
  rust::Vec<ZFCxxOutput> run(ZFCxxContext &context,
                             std::unique_ptr<ZFCxxState> &state);
} // namespace zenoh_flow
