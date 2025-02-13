# Project: Orbit
# Script: mansync.py
# Usage: python mansync.py
#
# Reads a TOML file to write various forms of documentation (markdown, rust).

import toml, os, sys
from license import NEW_HEADER

# --- Configurations -----------------------------------------------------------

# the entry-level command
PROGRAM = 'orbit'

CWD, _ = os.path.split(sys.argv[0])
# define the path to the TOML file
INPUT_TOML_PATH = "./docs/commands.toml"

# set the location of where to place the MD files
MD_OUTPUT_DIR = './docs/src/commands' # './mansync/md'
# set the location of where to place the Rust files
RS_OUTPUT_DIR = './src/commands/manuals' # './mansync/rs/man'
RS_HELP_OUTPUT_DIR = './src/commands/helps' # './mansync/rs/help

# --- Constants ----------------------------------------------------------------

# expected fields for each command
NAME = 'name'
SYNO = 'synopsis'
DESC = 'description'
OPTS = 'options'
ARGS = 'args'
EXPS = 'examples'
SUMM = 'summary'
HELP = 'help'
ALIAS = 'alias'

END = '\n'
SECT_END = END+END
INDENT = ' ' * 4

# --- Functions ----------------------------------------------------------------

def ucap(s: str) -> str:
    return s.capitalize()


def lcap(s: str) -> str:
    if s[1].islower() == True or s[1] == ' ':
        return s[0].lower() + s[1:]
    else:
        return s
    

def find_all_commands(table, prog):
    commands = []
    for k, _v in table.items():
        if k == prog:
            continue
        commands += [str(k)]
    return commands
 

def write_prog_quick_help(table, commands) -> str:
    command = PROGRAM
    MAX = 26

    qh = ucap(table[command][SUMM]) + '.'
    qh += '\n\n' + 'Usage:' + '\n'
    qh += INDENT + table[command][SYNO] + '\n'
    qh += '\n' + 'Commands:' + '\n'
    for cmd in commands:
        field = INDENT + cmd + ('' if ALIAS not in table[cmd] else ', ' + table[cmd][ALIAS])
        qh += field
        qh += ' ' * (MAX - len(field)) + lcap(table[cmd][SUMM]) + '\n'
        pass
    qh += '\n' + 'Options:' + '\n'
    table = table[command]
    for opt in table[OPTS]:
        field = INDENT + opt
        qh += field
        if len(field)+2 >= MAX:
            qh += '\n' + ' ' * MAX
        else:
            qh += ' ' * (MAX - len(field))
        qh += lcap(table[OPTS][opt]) + '\n'
    return qh

    pass


def write_quick_help(table, command) -> str:
    MAX = 26   
    # handle for subcommand
    table = table[command]
    qh = ucap(table[SUMM]) + '.'
    qh += '\n\n' + 'Usage:' + '\n'
    qh += INDENT + table[SYNO] + '\n'
    if ARGS in table.keys():
        qh += '\n' + 'Arguments:' + '\n'
        for arg in table[ARGS]:
            field = INDENT + arg
            qh += field
            if len(field)+2 >= MAX:
                qh += '\n' + ' ' * 26
            else:
                qh += ' ' * (MAX - len(field))
            qh += lcap(table[ARGS][arg]) + '\n'
    if OPTS in table.keys():
        qh += '\n' + 'Options:' + '\n'
        for opt in table[OPTS]:
            field = INDENT + opt
            qh += field
            if len(field)+2 >= MAX:
                qh += '\n' + ' ' * MAX
            else:
                qh += ' ' * (MAX - len(field))
            qh += lcap(table[OPTS][opt]) + '\n'
    return qh


def is_populated(table, key: str) -> bool:
    '''
    Checks a TOML `table` for if the given `key` is a valid entry with some value.
    '''
    return key in table and table[key] != None


def write_md_manual(table, dest: str, command: str) -> int:
    '''
    Writes the TOML `table` for a particular `command` to the `dest` folder in
    Markdown format.
    '''
    table = table[command]
    
    # fill in intermediate directory structure
    os.makedirs(dest, exist_ok=True)
    
    path = dest+'/'+command+'.md'

    with open(path, 'w') as md:
        # title
        md.write('# __orbit '+command+'__')
        md.write(SECT_END)

        # name section
        md.write('## __NAME__')
        md.write(SECT_END)
        md.write(table[NAME]+' - '+table[SUMM])
        md.write(SECT_END)

        # synopsis section
        md.write('## __SYNOPSIS__')
        md.write(SECT_END)
        md.write('```\n'+table[SYNO]+'\n```')
        md.write(SECT_END)

        # description section
        md.write('## __DESCRIPTION__')
        md.write(SECT_END)
        md.write(table[DESC].strip())
        md.write(SECT_END)

        # options section
        if is_populated(table, OPTS) == True or is_populated(table, ARGS) == True:
            md.write('## __OPTIONS__')
            md.write(SECT_END)
            pass

        if is_populated(table, ARGS) == True:
            for opt in table[ARGS]:
                md.write('`'+opt+'`'+'  \n')
                md.write('      '+ucap(table[ARGS][opt]))
                md.write(SECT_END)
            pass
        if is_populated(table, OPTS) == True:
            for opt in table[OPTS]:
                md.write('`'+opt+'`'+'  \n')
                md.write('      '+ucap(table[OPTS][opt]))
                md.write(SECT_END)
            pass

        # examples section
        md.write('## __EXAMPLES__')
        md.write(SECT_END)
        md.write('```\n'+table[EXPS].strip()+'\n```')
        md.write(SECT_END)
        pass

    print('INFO: Markdown manual page available at:', path)
    return 1


def write_rs_manual(table, dest, command) -> int:
    '''
    Writes the TOML `table` for a particular `command` to the `dest` folder in
    Rust format.
    '''
    table = table[command]
    
    # fill in intermediate directory structure
    os.makedirs(dest, exist_ok=True)

    # check if added to mod.rs
    mod_exists = False
    with open(dest+'/'+'mod.rs', 'r') as mod:
        mod_exists = mod.read().count('pub mod '+command+';') > 0
    if mod_exists == False:
        with open(dest+'/'+'mod.rs', 'a') as mod:
            mod.write('pub mod '+command+';')
        pass
    
    path = dest+'/'+command+'.rs'

    with open(path, 'w') as rs:
        # write the header
        rs.write(NEW_HEADER)
        # comment
        rs.write('// This manual page was automatically generated from the mangen.py tool.'+END)
        # variable declaration
        rs.write('pub const MANUAL: &str = r#"NAME'+END)

        # name section 
        rs.write(INDENT+table[NAME]+ ' - '+table[SUMM])
        rs.write(SECT_END)

        # synopsis section
        rs.write('SYNOPSIS'+END)
        rs.write(INDENT+table[SYNO])
        rs.write(SECT_END)

        # description section
        rs.write('DESCRIPTION'+END)
        lines = table[DESC].strip().splitlines()
        for line in lines:
            # flip any grave ticks to single quotes
            rs.write(INDENT+line.replace('`', '\'')+END)
        rs.write(END)

        # options section
        if is_populated(table, OPTS) == True or is_populated(table, ARGS) == True:
            rs.write('OPTIONS'+END)
            pass

        if is_populated(table, ARGS) == True:
            for opt in table[ARGS]:
                rs.write(INDENT+opt+END)
                rs.write(INDENT+INDENT+ucap(table[ARGS][opt])+END+END)
            pass
        if is_populated(table, OPTS) == True:
            for opt in table[OPTS]:
                rs.write(INDENT+opt+END)
                rs.write(INDENT+INDENT+ucap(table[OPTS][opt])+END+END)
            pass

        # examples section
        rs.write('EXAMPLES'+END)
        lines = table[EXPS].strip().splitlines()
        for line in lines:
            rs.write(INDENT+line+END)
        rs.write('"#;\n')
        pass

    print('INFO: Rust manual page available at:', path)
    return 1


def write_rs_help(ptable, dest, command, footer=True) -> int:
    '''
    Writes the TOML `table` for a particular `command` to the `dest` folder in
    Rust format as quick help.
    '''
    # fill in intermediate directory structure
    os.makedirs(dest, exist_ok=True)

    # check if added to mod.rs
    mod_exists = False
    with open(dest+'/'+'mod.rs', 'r') as mod:
        mod_exists = mod.read().count('pub mod '+command+';') > 0
    if mod_exists == False:
        with open(dest+'/'+'mod.rs', 'a') as mod:
            mod.write('pub mod '+command+';'+END)
        pass

    path = dest+'/'+command+'.rs'

    with open(path, 'w') as rs:
        # write the header
        rs.write(NEW_HEADER)
        # comment
        rs.write('// Automatically generated from the mansync.py script.'+END)
        # variable declaration
        rs.write('pub const HELP: &str = r#"')
        # quick help body from table
        if command == PROGRAM:
            quick_help = write_prog_quick_help(ptable, find_all_commands(ptable, PROGRAM))
            rs.write(quick_help.strip())
            rs.write(SECT_END+"Use 'orbit help <command>' for more information about a command.")
            pass
        else:
            quick_help = write_quick_help(ptable, command)
            rs.write(quick_help.strip())
            rs.write(SECT_END+"Use 'orbit help "+command+"' to read more about the command.")
            pass
        rs.write('"#;\n')
        pass

    print('INFO: Rust help page available at:', path)
    return 1


# --- Application Code ---------------------------------------------------------

def main():
    # open the manual data
    data = toml.load(INPUT_TOML_PATH)

    COMMANDS = find_all_commands(data, PROGRAM)
    
    score = 0
    total = 3 * len(COMMANDS) + 1

    # compile all documentation
    for cmd in COMMANDS:
        if cmd not in data:
            print("ERROR: Missing command in TOML file: '"+cmd+"'")
            print()
            continue
        # output the markdown files
        score += write_md_manual(data, MD_OUTPUT_DIR, cmd)
        # output the rust files
        score += write_rs_manual(data, RS_OUTPUT_DIR, cmd)
        # output the help files
        score += write_rs_help(data, RS_HELP_OUTPUT_DIR, cmd)
        print()
        pass

    score += write_rs_help(data, RS_HELP_OUTPUT_DIR, PROGRAM, footer=False)

    print('DOCUMENTATION SCORE:', score, '/', total)
    pass


if __name__ == '__main__':
    main()