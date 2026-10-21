<!DOCTYPE html>
<html>
<body class="orders">
<?php
namespace Shop\Orders;

use Shop\Model\Order;

/**
 * Keeps the orders of one customer.
 * @param string $owner who owns the book
 */
#[Immutable]
final class OrderBook implements \Countable
{
    const MAX_ORDERS = 100;
    private array $orders = [];

    public function __construct(private string $owner) {}

    public function add(Order $order, int ...$tags): static
    {
        if (count($this->orders) >= self::MAX_ORDERS) {
            throw new \OverflowException("Book of {$this->owner} is full\n");
        }
        $total = $order->total() * 1.2; // with tax
        $this->orders[] = ['id' => $order->id, 'total' => $total];
        return $this;
    }

    public function count(): int { return count($this->orders); /* all of them */ }
}

function render(OrderBook $book): string
{
    $note = <<<EOT
        Total: {$book->count()}
        EOT;
    return sprintf('%d orders', $book->count()) . OrderBook::summary(true, null) . $note;
}
?>
  <h1>Orders of <?= htmlspecialchars($owner) ?></h1>
</body>
</html>
