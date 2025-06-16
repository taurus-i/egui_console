#!/usr/bin/env python3

# This is a sample Python script for the terminal demo
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def distance_from_origin(self):
        """Calculate distance from origin"""
        import math
        return math.sqrt(self.x ** 2 + self.y ** 2)

def main():
    # Create a point
    point = Point(3, 4)

    # Calculate and display distance
    distance = point.distance_from_origin()
    print(f"Distance from origin: {distance}")

    # Loop example
    numbers = [1, 2, 3, 4, 5]
    total = 0
    for num in numbers:
        total += num

    print(f"Sum of numbers: {total}")

    # Conditional
    if total > 10:
        print("That's a big sum!")
    else:
        print("The sum is small.")

if __name__ == "__main__":
    main()
