%% Shapes on the plane
-module(shapes).
-moduledoc "Areas and labels of shapes.".
-behaviour(gen_server).
-export([area/1, total/1, start_link/0]).
-export_type([shape/0]).

-define(PI, 3.14159).
-record(rect, {width = 0 :: number(), height = 0 :: number()}).
-type shape() :: {circle, number()} | #rect{}.

-doc "The area of a shape.".
-spec area(shape()) -> number().
area({circle, Radius}) ->
    ?PI * Radius * Radius;
area(#rect{width = W, height = H}) when W > 0 ->
    W * H.

total(Shapes) ->
    Areas = lists:map(fun area/1, Shapes),
    io:format("~s: ~p~n", [?MODULE, Areas]),
    lists:sum([A || A <- Areas, A >= 0.5]).

start_link() ->
    Pid = spawn(fun() -> loop(#{count => 0, initial => $a}) end),
    Pid ! {self(), <<"ready">>},
    {ok, Pid}.

loop(State) ->
    receive
        {From, stop} -> From ! stopped;
        _Other -> loop(State)
    after 1000 ->
        timeout
    end.
