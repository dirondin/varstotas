# Extensions: plugin header parser errors, plugin dialogs, material plugins, the Extensions menu.

## Plugin header errors (shown to plugin authors)
# $label is a parameter label; $value is the text entered
ext-param-invalid = { $label }: not a valid value: { $value }
# $text is the @param line
ext-param-needs-default = @param needs `name = default`: { $text }
ext-param-bad-name = bad parameter name `{ $key }`
ext-param-unclosed-quote = unclosed quote: { $text }
ext-param-unknown-type = unknown parameter type `{ $ty }` (length, int, number, text, bool)
ext-pick-needs-number = @pick needs a number of points: { $text }
ext-pick-range = @pick: 1 to 16 points
ext-pick-default = Click { $n } point(s)
ext-select-default = Select objects
ext-empty-name = empty @name

## Material plugins
ext-mat-plugin-failed = the plugin failed
ext-mat-no-answer = the plugin printed no answer
# $error is the JSON parser's message; $answer is the start of the plugin's answer
ext-mat-not-json = its answer is not JSON ({ $error }): { $answer }
ext-mat-catalogue-array = catalogue() must return an array of materials
ext-mat-no-home = no home folder
ext-mat-default-name = Material
ext-mat-import-nothing = import_material() returned nothing
ext-mat-spec-object = a material spec must be an object

## Extensions menu and plugin dialogs
ext-script-undo = Script
# $error is the plugin header parser's message
ext-header-error = header: { $error }
ext-no-plugins = No plugins
# $n is the number of plugins
ext-failed-to-load = { $n } plugin(s) failed to load…
ext-reload = Reload Plugins
ext-open-folder = Open Plugins Folder
ext-script-gone = the script is gone
# $name is the plugin's or script's name (log lines)
ext-run-script = script { $name }
ext-run-plugin = plugin { $name }
ext-plugin-failed = plugin failed
ext-done = { $name } done
ext-pick-points = Pick Points
ext-select-objects = Select Objects
ext-run = Run
