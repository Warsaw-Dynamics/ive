module Shop

export OrderBook, total

const MAX_ORDERS = 100

#=
Orders are kept in memory;
a book is mutable.
=#
mutable struct OrderBook{T<:Real}
    owner::String
    prices::Vector{T}
end

"""
    total(book; tax=0.2)

Sums the prices of `book`, with tax.
"""
function total(book::OrderBook, extra...; tax=0.2)
    net = sum(book.prices) + length(extra)  # without tax
    net > MAX_ORDERS && @warn "large book" net
    return net * (1 + tax)
end

add!(book, price) = push!(book.prices, price)

book = OrderBook("ann", [10.0, 2.5, 0x1f])
for (i, price) in enumerate(book.prices)
    println("Order $i: $(price * 2)\n")
end
status = isempty(book.prices) ? :empty : :open
nothing
end
