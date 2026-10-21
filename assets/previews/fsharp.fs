namespace Ive.Preview

open System
open System.Collections.Generic

/// A shape with an area.
type Shape =
    | Circle of radius: float
    | Square of side: float

(* Block comment *)
[<Literal>]
let MaxSize = 0x100

[<AbstractClass>]
type Counter(start: int) =
    let mutable count = start
    member this.Count = count
    member this.Add(step: int) = count <- count + step
    static member Zero = Counter(0)

type Point = { X: int; Y: int }

module Geometry =
    // line comment
    let area shape =
        match shape with
        | Circle r -> Math.PI * r * r
        | Square s when s > 0.0 -> s * s
        | _ -> 0.0

    let rec sum (values: float list) : float =
        match values with
        | [] -> 0.0
        | head :: tail -> head + sum tail

    let describe (name: string) =
        let total = [ Circle 1.5; Square 2.0 ] |> List.map area |> sum
        let point = { X = 1; Y = 2 }
        printfn "%s: %.2f at %A\n" name total (point.X, point.Y)
#if DEBUG
        printfn $"Debug {name}"
#endif
        if total > 3.0 && not (String.IsNullOrEmpty name) then Some 'x' else None
