#nullable enable
using System;
using System.Collections.Generic;

namespace Ive.Preview;

/// <summary>A shape with an area.</summary>
public interface IShape
{
    double Area { get; }
}

/* Block comment */
[Serializable]
public sealed class Square<T> : IShape where T : struct
{
    private const int MaxSide = 0x100;
    private readonly double side;
    public static int Count;

    public Square(double side) => this.side = side;

    public double Area => side * side;

    public override string ToString() => $"Square {side:F2} of {Count}\n";
}

public enum Color { Red, Green }

public record Point(int X, int Y);

public readonly struct Size { public readonly int Width; }

internal static class Program
{
    private static async Task<int> Main(string[] args)
    {
        // line comment
        var shapes = new List<IShape> { new Square<int>(2.5) };
        int total = 0;
        foreach (var shape in shapes)
        {
            total += (int)shape.Area;
        }
        string path = @"C:\temp";
        char tab = '\t';
        Console.WriteLine(Describe(name: "box", total) ?? path + tab);
        await Task.Delay(10);
        return total > 1 && args.Length == 0 ? Square<int>.Count : -1;
    }

    private static void Spin()
    {
    retry:
        if (Square<int>.Count++ < 3) goto retry;
    }

    private static string Describe(string name, int count) => name is null ? "none" : $"{name}: {count}";
}
