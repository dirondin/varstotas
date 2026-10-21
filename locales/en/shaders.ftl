# Shader editor and the shader reference.

## Bundled presets
shader-preset-wood = Wood
shader-preset-water = Water
shader-preset-reeded-glass = Reeded Glass
shader-preset-glow = Glow

## Completion and call hints (descriptions of the helpers, WGSL built-ins and Surface fields)
shader-fn-surface = the shader: return the changed surface
shader-fn-hash3 = pseudo-random value 0..1 per point
shader-fn-noise = smooth value noise 0..1
shader-fn-fbm = layered noise 0..1 (clouds, marble, warping)
shader-fn-srgb = a colour picked in sRGB (0..1) → linear
shader-fn-hsv = hue / saturation / value (0..1) → linear colour
shader-fn-tangent_of = a unit vector across n
shader-fn-bump = n tilted by the slope of a height field
shader-fn-abs = absolute value
shader-fn-min = the smaller
shader-fn-max = the larger
shader-fn-clamp = x limited to lo..hi
shader-fn-saturate = x limited to 0..1
shader-fn-mix = a + (b − a)·t
shader-fn-step = 0 below edge, 1 from it
shader-fn-smoothstep = smooth 0→1 between lo and hi
shader-fn-fract = x − floor(x)
shader-fn-floor = round down
shader-fn-ceil = round up
shader-fn-round = nearest whole number
shader-fn-sign = −1, 0 or 1
shader-fn-sqrt = square root
shader-fn-pow = x to the power y
shader-fn-exp = e to the power x
shader-fn-log = natural logarithm
shader-fn-sin = sine (radians)
shader-fn-cos = cosine (radians)
shader-fn-tan = tangent (radians)
shader-fn-atan2 = angle of (x, y), radians
shader-fn-length = vector length
shader-fn-distance = length(a − b)
shader-fn-normalize = unit vector along v
shader-fn-dot = dot product
shader-fn-cross = cross product of two vec3
shader-fn-reflect = i mirrored about the normal n
shader-fn-select = t when cond, else f
shader-fn-vec2 = a 2-vector
shader-fn-vec3 = a 3-vector
shader-fn-vec4 = a 4-vector
shader-surface-base = linear base colour
shader-surface-alpha = opacity
shader-surface-metallic = 0 = dielectric, 1 = metal
shader-surface-roughness = 0 = mirror, 1 = matte
shader-surface-transmission = glass: 1 = light passes through
shader-surface-emission = light given off, linear, strength included
shader-surface-normal = world normal, unit, facing the viewer
shader-surface-pos = model position, mm (z = up)
shader-surface-uv = planar texture coordinates, mm
shader-surface-view = unit direction toward the eye
shader-surface-time = seconds
shader-param-tag = a slider of the material editor, read as p.name (at most 8)
