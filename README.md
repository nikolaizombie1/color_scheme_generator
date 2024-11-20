# color_scheme_generator

Quickly generate color schemes for waybar from an image.

color_scheme_generator is a command line utility used to analyze images
and generate color themes from them given a path to an image.

This command line utility behaves like a standard UNIX utility where the path to the image can be either piped in or sent a command line argument.

The intended purpose of this application is to automatically create color themes for
Waybar, but it can be used used for the bar in AwesomeWM or other applications to theme based on the on an image.
This utility has a cache for the image analysis. This means that once an image has been analyzed once, the result will be saved in the cache and when an image is analyzed again, the results will be returned instantly.

# Usage Examples
```bash
echo PATH_TO_IMAGE | color_scheme_generator
```
```bash
color_scheme_generator PATH_TO_IMAGE
```

# Output Formats
color_scheme_generator can output to 3 different output formats all of which give an RGB8 value in the form of "bar_color", "workspace_color" and "text_color":
1. JSON
```json
[{"bar_color":{"red":222,"green":186,"blue":189},"workspace_color":{"red":33,"green":69,"blue":66},"text_color":{"red":255,"green":255,"blue":255}}]
```
2. YAML
```yaml
- bar_color:
    red: 222
    green: 186
    blue: 189
  workspace_color:
    red: 33
    green: 69
    blue: 66
  text_color:
    red: 255
    green: 255
    blue: 255
```
3. Text
```
DEBABD,214542,FFFFFF
```
The text output has the format of `BAR_COLOR,WORKSPACE_COLOR,TEXT_COLOR`.
