from PIL import Image, ImageDraw, ImageFont
import os

#def create_icon(text, size=(100, 100), bg_color=(100, 200, 100), text_color=(0, 0, 0), border_color=(0, 0, 0), border_width=2):
def create_icon(text, size=(100, 100), bg_color=(120, 180, 120), text_color=(0, 0, 0), border_color=(0, 0, 0), border_width=2):
    # Create a new image with a light green background
    image = Image.new('RGB', size, bg_color)
    draw = ImageDraw.Draw(image)

    # Draw the border
    for i in range(border_width):
        draw.rectangle([i, i, size[0]-i-1, size[1]-i-1], outline=border_color)

    # Use a default font
    try:
        font = ImageFont.truetype("arial.ttf", 32)
    except IOError:
        font = ImageFont.load_default()

    # Get text size
    text_width, text_height = draw.textsize(text, font=font)

    # Calculate position to center the text
    position = ((size[0] - text_width) / 2, (size[1] - text_height) / 2)

    # Draw the text
    draw.text(position, text, font=font, fill=text_color)

    return image

def main():
    speeds = [0, 1, 3, 10, 50, 200]
    icons = ['pause', '1x', '3x', '10x', '50x', '200x']

    # Create 'icons' directory if it doesn't exist
    if not os.path.exists('icons'):
        os.makedirs('icons')

    for speed, icon_name in zip(speeds, icons):
        # Create the icon
        if speed == 0:
            text = "||"  # Pause symbol
        elif speed == 1:
            text = "1x"   # Play symbol
        else:
            text = f"{speed}x"
        
        icon = create_icon(text)

        # Save the icon
        filename = f"icons/{icon_name.replace(' ', '_')}.png"
        icon.save(filename)
        print(f"Created icon: {filename}")

if __name__ == "__main__":
    main()