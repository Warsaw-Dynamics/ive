{-# LANGUAGE ScopedTypeVariables #-}
{- Block comment -}
module Shapes (Shape (..), area, describe) where

import qualified Data.Map as Map

-- | A shape on the plane, with a Haddock comment.
data Shape
  = Circle Double
  | Rect { width :: Double, height :: Double }
  deriving (Show, Eq)

class Describable a where
  describe :: a -> String

instance Describable Shape where
  describe (Circle r) = "circle of radius " ++ show r
  describe shape = "rectangle\t" ++ show (area shape)

-- Line comment
area :: Shape -> Double
area (Circle r) = pi * r * r
area Rect {width = w, height = h} = w * h

scaled :: forall a. Num a => a -> [a] -> [a]
scaled factor xs = map (* factor) xs

main :: IO ()
main = do
  let shapes = [Circle 1.5, Rect 2 3]
      table = Map.fromList (zip [1 :: Int ..] shapes)
  mapM_ (putStrLn . describe) shapes
  print (Map.size table, 'x', True)
