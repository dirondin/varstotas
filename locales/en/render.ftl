render-dof-view = As view
render-dof-off = Off
render-dof-on = On

## Quality presets and image sizes (the ids stay English)

render-preset-draft = Draft
render-preset-preview = Preview
render-preset-final = Final
render-preset-best = Best
render-size-hd = HD 1280 × 720
render-size-full-hd = Full HD 1920 × 1080
render-size-qhd = QHD 2560 × 1440
render-size-4k = 4K 3840 × 2160
render-size-square = Square 1080
render-size-portrait = Portrait 1080 × 1350

## Months (sun date)

render-month-jan = Jan
render-month-feb = Feb
render-month-mar = Mar
render-month-apr = Apr
render-month-may = May
render-month-jun = Jun
render-month-jul = Jul
render-month-aug = Aug
render-month-sep = Sep
render-month-oct = Oct
render-month-nov = Nov
render-month-dec = Dec

## Export and shader errors

render-export-empty = image size must be at least 1 x 1 pixels
# $w and $h are the requested size, $max the limit per side.
render-export-too-large = image size { $w } x { $h } is too large (at most { $max } pixels per side)
render-export-gpu = GPU error while exporting the image: { $error }
render-shader-clash = does not compile with the other shaders: { $error }

## Render window

render-title = Render
render-current-view = Render the current view
render-stop = Stop
render-continue = Continue
render-more-samples = More Samples
render-more-samples-tip = Double the samples
render-save-png = Save PNG…
render-save-exr = Save EXR…
render-save-exr-tip = Linear light, 32-bit float
render-all-scenes = Render All Scenes…
# $error is the reason.
render-failed = Render failed: { $error }
render-error = Render: { $error }
# $n is the number of scenes, $folder where they were saved.
render-scenes-done = Rendered { $n } scene(s) to { $folder }
# $written scenes are saved, scene $next is being rendered.
render-scenes-progress = Rendering scenes: { $written } saved, scene { $next } in progress
# $path is the saved file.
render-saved = Saved { $path }
# $secs is a number with one decimal.
render-status-running = { $samples } / { $max } samples · { $secs } s
# $denoised is "yes" or "no".
render-status-done = Done: { $samples } samples in { $secs } s{ $denoised ->
        [yes] , denoised
       *[no] {""}
    }
render-status-stopped = Stopped at { $samples } samples · { $secs } s
render-undo-lights = Edit Lights
render-undo-settings = Render Settings

## Lights

render-lights = Lights ({ $n })
render-add-point = + Point
render-add-spot = + Spot
render-add-area = + Area
render-on = On
render-light-point = point
render-light-spot = spot
render-light-area = area
render-light-name = Name
render-light-type = Type
render-light-position = Position
render-light-to-centre = To view centre
render-light-to-centre-tip = Move it 2 m above the camera target
render-light-to-camera = To camera
render-light-aim-at-target = Aim at target
render-light-aim-at-target-tip = Shine at the view's target
render-light-aim = Aim
render-colour = Colour
render-light-power = Power
render-light-radius = Radius
render-light-radius-tip = Bulb size: softer shadows
render-light-cone = Cone
render-light-cone-angle = Cone angle
render-light-soft = Soft
render-light-soft-edge = Soft edge
render-size = Size

## Settings

render-sec-image = Image
render-sec-sampling = Sampling
render-sec-camera = Camera
render-sec-light = Light
render-sec-colour = Colour
render-quality = Quality
# $samples and $bounces are numbers.
render-quality-tip = { $samples } samples, { $bounces } bounces
render-match-view = Match view
render-match-view-tip = The view's proportions at this height
render-size-custom = Custom
render-transparent = Transparent background
render-transparent-tip = The sky and the ground behind the model are transparent in the PNG (they still light it)
render-samples = Samples
render-time-limit = Time limit
render-time-limit-tip = 0 = until the samples are done
render-bounces = Bounces
render-clamp = Clamp
render-clamp-tip = Brightest indirect light of a sample: lower kills fireflies, higher keeps bright bounce light; 0 = off
render-denoiser = Denoiser
render-denoiser-off = Off
render-denoiser-builtin = Built-in
render-denoiser-builtin-tip = Edge-aware filter
render-denoiser-ai = AI (Open Image Denoise)
render-denoiser-install = Install Open Image Denoise: brew install open-image-denoise
render-dof = Depth of field
render-dof-tip = As view: the Effects' depth of field when it is on
# $mm is the equivalent focal length.
render-lens = { $mm } mm lens
render-lens-tip = The view's field of view on a 35 mm camera
render-aperture = Aperture
render-aperture-tip = Smaller f-number = shallower focus
render-focus = Focus
render-focus-target = Target
render-focus-target-tip = Focus on the view's target (orbit centre)
render-sun = Sun
render-sun-tip = Sun strength (its direction: Shadows)
render-sky = Sky
render-sky-tip = Environment light
render-rotation = Rotation
render-rotation-tip = Turns the sky / HDRI
render-exposure = Exposure
render-env-exposure-tip = The environment's exposure (view and render)
render-exposure-tip = Added to the environment's (render only)
render-tone-curve = Tone curve
# $name is the environment's tone curve.
render-tone-environment = Environment: { $name }
render-white-balance = White balance
render-white-balance-tip = 6500 K = neutral; higher is warmer
render-tint = Tint
render-tint-tip = Green (−) .. magenta (+)
render-saturation = Saturation
render-contrast = Contrast
render-vignette = Vignette
render-reset = Reset
render-reset-tip = Back to the default colour settings
render-shader-define-surface = define fn surface(s: Surface, p: Params) -> Surface
# $line is the line in the user's shader code, $msg the compiler's message.
render-shader-line = line { $line }: { $msg }
