#include <iostream>
#include <string>

size_t foo(size_t x, const std::string& s) {
    // const std::string s = "testing";

    const auto a = s.find('s');
    const auto b = a != std::string::npos ? s.find('e') : std::string::npos;
    const auto c = b != std::string::npos ? s.find('i') : std::string::npos;

    return b != std::string::npos ? x + a + b + c : b != std::string::npos;
}

int main() {
    std::string s;

    std::cin >> s;

    size_t x = 0;
    for (size_t i = 0; i < 100'000'000; ++i) {
        const size_t y = foo(x, s);
        if (y != std::string::npos) {
            x = y;
        }
    }

    std::cout << x << std::endl;

    return 0;
}
