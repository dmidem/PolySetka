#include <iostream>
#include <numeric>
#include <vector>

using Real = double;

Real foo(Real x, const std::vector<Real>& s) {
    return std::accumulate(
        s.begin(), s.end(), x,
        [](const Real sum, const Real si) { return sum + si * si; });
}

int main() {
    std::vector<Real> s{1.5, 2.5, 3.5, 4.5, 5.5, 1.5, 2.5, 3.5, 4.5, 5.5,
                        1.5, 2.5, 3.5, 4.5, 5.5, 1.5, 2.5, 3.5, 4.5, 5.5};

    Real x = Real(0);

    std::cin >> x;

    for (size_t i = 0; i < 100'000'000; ++i) {
        x = foo(x, s);
    }

    std::cout << std::fixed << x << std::endl;

    return 0;
}
