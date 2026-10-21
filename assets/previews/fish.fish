#!/usr/bin/env fish
# Greets everyone in the list and logs to a file
set -l names alice bob carol
set --global retries 3
set ratio 0.75

function greet --description "Say hello"
    for name in $names
        if test (count $argv) -gt 0; and not contains $name $argv
            continue
        end
        echo "Hello, $name!\n" >> greetings.log
    end
    return 0
end

switch (uname)
    case Linux Darwin
        greet $names[1..2] {a,b}
    case '*'
        printf 'unknown system\n' 2>&1
end

while test $retries -gt 0
    git fetch origin; or set retries (math $retries - 1)
    mkdir -p ~/cache && ls *.fish | sort
end
