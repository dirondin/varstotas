# The Realistic style: Environment card, physical material properties, shader editor, effects.

realistic-undo-material = Edit Material
# $path is a file, $error the reason.
realistic-cant-use = Can't use { $path }: { $error }
realistic-cant-read = Can't read { $path }: { $error }
realistic-used-by = Used by the Realistic style
realistic-load = Load…
realistic-clear = Clear
realistic-reset = Reset
realistic-preset = Preset
realistic-paint-first = Paint something with the material first

## Environment

realistic-background = Background
realistic-sky-sky = Sky
realistic-sky-color = Color
realistic-sky-hdri = HDRI
realistic-sky-physical = Physical sky
realistic-sky-physical-tip = Daylight for the sun's position in the Shadows card
realistic-haze = Haze
realistic-haze-tip = Turbidity: 2 = clear blue sky, 10 = hazy; the sun's time and date are in the Shadows card
realistic-hdri-none = none (sky)
realistic-hdri-tip = An equirectangular panorama (.hdr)
realistic-rotation = Rotation
realistic-intensity = Intensity
realistic-sun = Sun
realistic-sun-tip = Sun strength; its direction comes from the Shadows card when shadows are on
realistic-exposure = Exposure
realistic-tone-mapping = Tone mapping
realistic-tonemap-aces = ACES (filmic)
realistic-tonemap-agx = AgX (soft)
realistic-tonemap-none = None
realistic-show-env = Show the environment behind the model
realistic-effects = Effects

## Physical properties

realistic-physical = Physical (Realistic)
realistic-preset-matte = Matte
realistic-preset-satin = Satin
realistic-preset-glossy = Glossy
realistic-preset-metal = Metal
realistic-preset-chrome = Chrome
realistic-preset-glass = Glass
realistic-preset-light = Light
realistic-metallic = Metallic
realistic-metallic-tip = 0 = paint, plastic, wood; 1 = bare metal
realistic-roughness = Roughness
realistic-roughness-tip = 0 = mirror-like, 1 = matte
realistic-glass = Glass
realistic-glass-tip = How much light passes through
realistic-ior = IOR
realistic-ior-tip = Index of refraction (glass 1.5, water 1.33)
realistic-emission = Emission
realistic-emission-tip = Light the surface gives off: colour and strength
realistic-coat-sheen = Coat, Sheen, Edges
realistic-clearcoat = Clear coat
realistic-clearcoat-tip = A glossy varnish layer: lacquered wood, car paint, glazed tiles
realistic-coat-rough = Coat rough
realistic-coat-rough-tip = Roughness of the varnish (0 = glossy)
realistic-sheen = Sheen
realistic-sheen-tip = A soft glow at grazing angles: velvet, cloth, upholstery
realistic-sheen-tint = Sheen tint
realistic-sheen-tint-tip = 0 = white sheen, 1 = the colour of the material
realistic-anisotropy = Anisotropy
realistic-anisotropy-tip = Stretched highlights along the texture (+) or across it (−): brushed metal
realistic-anti-tiling = Anti-tiling
realistic-anti-tiling-tip = Breaks up the visible repetition of the textures
realistic-bevel = Bevel, mm
realistic-bevel-tip = Rounds the edges of faces in the render (path tracing only)
realistic-shader = Shader
realistic-shader-tip = A WGSL surface program (docs/SHADERS.md): procedural patterns, waves, glow
realistic-shader-none = None
realistic-shader-custom = Custom
realistic-shader-edit = Edit…
realistic-shader-edit-tip = Write or change the shader
realistic-map-normal = Normal
realistic-map-roughness = Roughness
realistic-map-metallic = Metallic
realistic-map-occlusion = Occlusion
realistic-map-emission = Emission
realistic-normal-strength = Normal strength
realistic-remove-physical = Remove physical properties

## Shader editor

# $material is the material's name.
realistic-shader-window = Shader — { $material }
realistic-start-from = Start from…
realistic-start-from-tip = Replace the code with this shader
realistic-checker-new = Checker (new)
realistic-reference = Reference
realistic-reference-tip = Surface fields, helpers and built-ins
realistic-compiles = Compiles; the material uses it
realistic-ref-surface = Surface (s, o)
realistic-ref-parameters = Parameters
realistic-ref-functions = Functions

## Effects

realistic-fx-off = Off
realistic-fx-fast = Fast
realistic-fx-balanced = Balanced
realistic-fx-quality = Quality
realistic-fx-soft-shadows = Soft shadows
realistic-fx-soft-shadows-tip = Shadows blur away from what casts them
realistic-fx-sun-size = Sun size
realistic-fx-ao = Ambient occlusion
realistic-fx-ao-tip = Darker creases and contact shadows
realistic-fx-radius = Radius, mm
realistic-fx-strength = Strength
realistic-fx-reflections = Reflections
realistic-fx-reflections-tip = The model mirrored in glossy surfaces (screen space)
realistic-fx-bloom = Bloom
realistic-fx-bloom-tip = Bright light glows
realistic-fx-threshold = Threshold
realistic-fx-dof = Depth of field
realistic-fx-dof-tip = Blur in front of and behind the focus distance
realistic-fx-blur = Blur
realistic-fx-focus = Focus, mm
realistic-fx-focus-tip = 0 = the point at the centre of the view
realistic-fx-fog = Fog
realistic-fx-fog-tip = Haze growing with distance
realistic-fx-fog-distance = Distance, mm
realistic-fx-fog-height = Height, mm
realistic-fx-fog-height-tip = 0 = even fog; else it thins out over this height
realistic-vignette = Vignette
realistic-style = Style
realistic-stylize-photo = Photo
realistic-stylize-pencil = Pencil
realistic-stylize-watercolor = Watercolor
realistic-stylize-technical = Technical
realistic-fx-clay = Clay
realistic-fx-clay-tip = Every material as plain clay (a massing model)
realistic-fx-high-quality = High quality
realistic-fx-high-quality-tip = More samples for soft shadows, occlusion and reflections
