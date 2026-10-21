module Shapes exposing (Shape(..), area, view)

import Html exposing (Html, text)
import List as L

{-| A shape on the plane.
-}
type Shape
    = Circle Float
    | Rect { width : Float, height : Float }

type alias Labelled a =
    { label : String, item : a }

-- The area of a shape
area : Shape -> Float
area shape =
    case shape of
        Circle radius ->
            pi * radius * radius * 1.0

        Rect { width, height } ->
            width * height

{- Joins the labels -}
view : List (Labelled Shape) -> Html msg
view shapes =
    let
        total = L.sum (L.map (\s -> area s.item) shapes)
    in
    text ("Total:\t" ++ String.fromFloat total ++ [ 'x' ] |> List.length |> String.fromInt)
