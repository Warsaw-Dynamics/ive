require "json"

# Keeps the orders of one customer.
module Shop
  class OrderBook < Base
    MAX_ORDERS = 100
    @@instances = 0

    attr_reader :orders

    def initialize(owner, limit: MAX_ORDERS)
      @owner = owner
      @orders = []
      @@instances += 1
    end

    def add(order, *tags, **options, &block)
      raise ArgumentError, "full: #{@orders.size}\n" if @orders.size >= MAX_ORDERS
      total = order.total * 1.2
      @orders << { id: order.id, :total => total, "tags" => tags }
      notify(order) unless tags.empty?
      yield order if block_given?
      self
    end

    def self.parse(text) = text.scan(/\d+-[a-z]+/i).map { |code| code.upcase }
  end
end

=begin
Usage
=end
book = Shop::OrderBook.new("ann", limit: 10)
puts <<~REPORT
  Orders of #{book.owner}: #{book.orders.length}
REPORT
$stdout.puts [1, 2, 0x1F].sum; nil
