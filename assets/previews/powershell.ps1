<#
.SYNOPSIS
    Totals the orders of one customer.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $Owner,
    [double] $Tax = 0.2
)

$MaxOrders = 100 <# orders per book #>
enum Status { Open; Closed }

class OrderBook {
    [string] $Owner
    [System.Collections.ArrayList] $Orders = @()
    hidden [int] $Version = 1

    [double] Total() {
        return ($this.Orders | Measure-Object -Property Price -Sum).Sum
    }
}

function Get-Total([OrderBook] $Book) {
    $net = $Book.Total()  # without tax
    if ($net -gt $MaxOrders -and -not $env:QUIET) {
        Write-Warning "Book of $($Book.Owner) is large`n"
    }
    return $net * (1 + $Tax)
}

$book = [OrderBook]::new()
$book.Orders.Add(@{ Price = 0x10; Id = 'a-1'; State = [Status]::Open }) | Out-Null
:orders foreach ($order in $book.Orders) {
    if ($order.Id -eq 'skip') { continue }
    Write-Host "Order $($order.Id)"
}
$report = @"
Owner: $Owner
"@
Get-Total -Book $book; exit 0
