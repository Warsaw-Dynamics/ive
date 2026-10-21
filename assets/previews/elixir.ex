# Shapes on the plane
defmodule IVE.Shapes do
  @moduledoc """
  Areas and labels of shapes.
  """
  use Phoenix.Component
  alias IVE.Geometry, as: Geo

  @pi 3.14159
  @email ~r/^[\w.]+@\w+\.com$/
  @kinds ~w(circle rect)a

  @doc "The area of `shape`."
  def area({:circle, radius}), do: @pi * radius * radius
  def area(%{width: w, height: h}) when w > 0, do: w * h

  defp describe(shape, opts \\ []) do
    label = Keyword.get(opts, :label, "shape")
    "#{label}:\t#{area(shape)}"
  end

  def total(shapes) do
    shapes
    |> Enum.map(&area/1)
    |> Enum.sum()
    |> Geo.round(2)
  end

  def card(assigns) do
    ~H"""
    <div class="card" :if={@visible}>
      <.button phx-click="grow">{describe(@shape)}</.button>
    </div>
    """
  end

  def valid?(email), do: Regex.match?(@email, email) and not is_nil(email)
  def initials(user), do: [?I, user.name, 'chars', nil, __MODULE__]
  def double(list), do: Enum.map(list, fn x -> x * 2 end)
end
