complete -c fkp -f

complete -c fkp -n "__fish_use_subcommand" -a "help info check edit switch sync git"

complete -c fkp -n "__fish_use_subcommand" -s h -d "Help"
complete -c fkp -n "__fish_use_subcommand" -s i -d "Info"
complete -c fkp -n "__fish_use_subcommand" -s c -d "Check"
complete -c fkp -n "__fish_use_subcommand" -s e -d "Edit"

complete -c fkp -n "__fish_seen_subcommand_from sync" -xa "local" -d "Sync local entries"
complete -c fkp -n "__fish_seen_subcommand_from sync" -xa "keep" -d "Sync keep entries"

complete -c fkp -n "__fish_use_subcommand" -s S -d "Sync" -xa "local" -d "Sync local entries"
complete -c fkp -n "__fish_use_subcommand" -s S -d "Sync" -xa "keep" -d "Sync keep entries"

complete -c fkp -n "__fish_seen_subcommand_from git" -xa "push pull restore status"
complete -c fkp -n "__fish_use_subcommand" -s g -d "Git" -xa "push pull restore status"

complete -c fkp -n "__fish_seen_subcommand_from check" -x -l "verbose" -s V -d "verbose mode"
