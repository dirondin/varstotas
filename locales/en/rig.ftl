# Component parameters (rigs): the editor window and the Entity Info block.

rig-title = Component Parameters
rig-select-hint = Select a component (not a group) and choose Window → Component Parameters, or Edit Parameters… in Entity Info.
rig-state-has = — with parameters
rig-state-none = — no parameters yet: Apply adds them
rig-no-parts = The component has no parts to move: group its pieces inside it (a door, a drawer…) — open it and use Make Group.

## Entity Info block
# $name is the component's name; $n the number of copies
rig-info-copies = { $name } — { $n } copies
rig-no-params = No parameters yet
rig-reset = Reset
rig-reset-hint = Back to the default values
rig-edit = Edit Parameters…
rig-edit-hint = Window → Component Parameters: parameters, formulas, parts' motions, script
# $n is the number of motions
rig-info-summary = Component with parameters ({ $n } motion(s))

## Undo steps and messages
rig-undo-reset = Reset Parameters
rig-undo-edit = Edit Parameters
rig-undo-add = Add Parameters
rig-undo-remove = Remove Parameters
rig-select-rigged = Select components with parameters
rig-select-component = Select a component to give it parameters
rig-err-group = Groups can't have parameters: make it a component first (Edit → Make Component)
rig-err-generated = This component is built by a generator or a parametric group
rig-applied = Parameters applied
rig-removed = Parameters removed
rig-err-place-copy = Place a copy of the component to pick in the model
rig-pick-axis = Click two points on the axis of rotation
rig-pick-slide = Click two points: the direction of the slide
rig-err-same-points = The two points are the same

## Parameters table
rig-parameters = Parameters
rig-parameters-hint = The values each copy has (Entity Info shows them)
rig-col-name = Name
rig-col-label = Label
rig-col-type = Type
rig-col-min = Min
rig-col-max = Max
rig-col-step = Step
rig-col-default = Default
rig-col-options = Options
rig-as-drawn = (as drawn)
rig-remove-param = Remove the parameter
rig-add-param = + Parameter

## Formulas
rig-formulas = Formulas
rig-formulas-hint = Values computed from the parameters (Rhai expressions), usable by the motions and the script
rig-add-formula = + Formula

## Motions
rig-motions = Motions
rig-motions-hint = A part (a group / component directly inside) turned, slid, shown, painted or given a parameter value (Set parameter: a part that has parameters itself — a generated component such as a Draw → Shapes cylinder, or a component with parameters) by a parameter. Turn / slide: the parameter's range maps onto From..To (degrees / length); without a range, or with an expression, the value is the motion itself
rig-motion-rotate = Rotate
rig-motion-slide = Slide
rig-motion-show = Show / hide
rig-motion-paint = Paint
rig-motion-set = Set parameter
rig-parameter = (parameter)
rig-part-no-params = The part has no parameters
rig-part-param-hint = The part's own parameter set to the value
rig-by = by
rig-expression = (expression)
rig-from = from
rig-to = to
rig-remove-motion = Remove the motion
rig-axis-through = Axis through
rig-along = along
rig-direction = Direction
rig-pick = Pick…
rig-pick-hint = Click two points in the model
rig-axes-note = mm, the component's axes
rig-add-motion = + Motion

## Script
rig-script = Script
rig-script-hint = Optional Rhai code run after the motions inside the component: fn update(p) {"{"} … {"}"} with the whole script API (rotate, move, paint, show / hide, set_params, box…); child(name) / child(name, true) finds a part by name (deeply), part.min / max / size / bounds — its extents; p — the values and formulas
rig-add-script = + Script

## Buttons
rig-add = Add Parameters
rig-revert = Revert
rig-remove = Remove Parameters
rig-remove-hint = The copies show the component as drawn again
