(* Block comment *)
open Printf

(** A shape on the plane, with a doc comment. *)
type shape =
  | Circle of float
  | Rect of { width : float; height : float }
[@@deriving show]

type ('k, 'v) table = Empty | Bind of 'k * 'v * ('k, 'v) table

module Geometry = struct
  let pi = 3.14159

  let area = function
    | Circle r -> pi *. r *. r
    | Rect { width; height } -> width *. height

  let scale ~factor ?(offset = 0.) shape =
    match shape with
    | Circle r -> Circle ((r *. factor) +. offset)
    | Rect rect -> Rect { rect with width = rect.width *. factor }
end

exception Invalid_shape of string

let describe shape =
  let total = Geometry.area shape in
  if total < 0. then raise (Invalid_shape "negative")
  else sprintf "area %.2f\n" total

let () =
  let shapes = [ Circle 1.5; Rect { width = 2.; height = 3 |> float_of_int } ] in
  let big = Geometry.scale ~factor:2. (Circle 1.) in
  List.iter (fun s -> print_string (describe s)) (big :: shapes);
  print_endline (string_of_bool true ^ String.make 1 'x')
