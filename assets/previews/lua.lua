#!/usr/bin/env lua
--- Keeps the orders of one customer.
local OrderBook = {}
OrderBook.__index = OrderBook

local MAX_ORDERS <const> = 100

--[[ A book starts empty;
     orders are added with add. ]]
function OrderBook.new(owner)
  local self = setmetatable({}, OrderBook)
  self.owner = owner
  self.orders = { count = 0, last = nil }
  return self
end

function OrderBook:add(order, ...)
  if #self.orders >= MAX_ORDERS then
    error("full: " .. self.owner .. "\n")
  end
  table.insert(self.orders, order); self.orders.count = self.orders.count + 1
  return true
end

local total = function(book)
  local sum = 0
  for i, order in ipairs(book.orders) do
    if order.price ~= nil and not order.void then sum = sum + order.price * 1.2 end
  end
  goto done
  ::done::
  return sum
end

local book = OrderBook.new("ann")
book:add({ price = 0x10 })
print(string.format("%d", total(book)), book.orders[1])
