# fish completion script for gradient <https://github.com/mazznoer/gradient-rs>

function __gradient_is_set
    __fish_seen_argument -s p -l preset; or __fish_seen_argument -s c -l custom; or __fish_seen_argument -s f -l file
end

complete -c gradient -f

# preset gradient
# TODO preset name completion is not working
complete -c gradient -n 'not __gradient_is_set' -s p -l preset -x -a "(gradient --list-presets)" -d 'Preset gradient'

# custom gradient
complete -c gradient -n 'not __gradient_is_set' -s c -l custom -x -d 'Create custom gradient'
complete -c gradient -n '__fish_seen_argument -s c -l custom' -s i -l interpolation -x -a 'linear basis catmull-rom' -d 'Sets custom gradient interpolation mode'
complete -c gradient -n '__fish_seen_argument -s c -l custom' -s m -l blend-mode -x -a 'rgb linear-rgb hsv oklab' -d 'Sets custom gradient blending mode'
complete -c gradient -n '__fish_seen_argument -s c -l custom' -s P -l position -x -d 'Sets custom gradient color position'

# gradient file
# TODO file completion not working
complete -c gradient -n 'not __gradient_is_set' -s f -l file -x -a "(__fish_complete_suffix .svg; __fish_complete_suffix .ggr;)" -d 'Read gradients from files'
complete -c gradient -n '__fish_seen_argument -s f -l file' -l ggr-bg -x -d 'Sets GGR background color'
complete -c gradient -n '__fish_seen_argument -s f -l file' -l ggr-fg -x -d 'Sets GGR foreground color'
complete -c gradient -n '__fish_seen_argument -s f -l file' -l svg-id -x -d 'Pick one SVG gradient by ID'

complete -c gradient -n '__gradient_is_set' -s b -l background -x -d 'Sets background color'
complete -c gradient -n '__gradient_is_set' -l cb-color -x -d 'Sets checkerboard color'
complete -c gradient -n '__gradient_is_set' -s W -l width -x -d 'Sets gradient display width'
complete -c gradient -n '__gradient_is_set' -s H -l height -x -d 'Sets gradient display height'
complete -c gradient -n '__gradient_is_set' -s t -l take -x -d 'Get N colors evenly spaced accross gradient'
complete -c gradient -n '__gradient_is_set' -s s -l sample -x -d 'Get color(s) at specific position'
complete -c gradient -n '__fish_seen_argument -s t -l take; or __fish_seen_argument -s s -l sample; or __fish_seen_argument -s C -l convert' -s o -l format -x -a 'hex rgb rgb255 hsl hsv hwb' -d 'Sets output color format'

complete -c gradient -n '__fish_no_arguments' -s l -l list-presets -x -d 'Lists preset gradient names'
complete -c gradient -n '__fish_no_arguments' -s h -l help -d 'Display help and exit'
complete -c gradient -n '__fish_no_arguments' -s V -l version -d 'Display version information and exit'

