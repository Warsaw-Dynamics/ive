library(stats)

#' Totals the orders of one customer.
#' @param orders a data frame with a price column
#' @return the total, with tax
order_total <- function(orders, tax = 0.2, ...) {
  if (is.null(orders) || nrow(orders) == 0L) {
    return(NA)
  }
  net <- sum(orders$price, na.rm = TRUE)
  for (i in seq_len(nrow(orders))) {
    if (orders[[i, "void"]]) next
  }
  net * (1 + tax)
}

# A small sample
orders <- data.frame(price = c(10, 2.5, 1e3), void = c(FALSE, FALSE, TRUE))
total <- order_total(orders)
shown <- stats::median(orders$price) |> round(digits = 2)
cat(sprintf("Total: %.2f\n", total), shown, Inf, NULL)
square <- \(x) x^2
