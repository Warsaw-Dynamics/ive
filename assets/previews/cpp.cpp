#include <vector>
#include "spans.hpp"

#define LOG(message) std::clog << (message) << '\n'

namespace ive::colors {

/* A run of text in one colour. */
enum class Kind { Plain, Keyword, Comment };

template <typename T>
concept Measured = requires(T t) { t.size(); };

/// Spans of one line; `MAX_SPANS` bounds them.
class Line final {
public:
    static constexpr int MAX_SPANS = 0x100;

    explicit Line(std::string text) : text_(std::move(text)) {}
    ~Line() = default;

    [[nodiscard]] bool operator==(const Line &other) const noexcept { return text_ == other.text_; }

    template <Measured Range>
    void add(const Range &range, Kind kind = Kind::Plain);

private:
    std::string text_;
    std::vector<Kind> kinds_;
};

template <Measured Range>
void Line::add(const Range &range, Kind kind) {
    for (const auto &item : range) {
        if (kinds_.size() < MAX_SPANS && item != nullptr) kinds_.push_back(kind);
    }
    LOG(R"(added)" "\t" + std::to_string(range.size() * 1.5));
}

int total_lines = 0;

}  // namespace ive::colors

int main(int argc, char *argv[]) {
    auto line = ive::colors::Line{"int x = 1;"};
    double ratio = argc / 2.0;
    if (ratio > 1) goto done;
    line.add(std::vector<int>{1, 2, 3});
done:
    return line == line ? 0 : -1;
}
