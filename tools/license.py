# Updates the license header on all source files.
# Usage: python license.py

import glob

NEW_HEADER = '''//
//  Copyright (C) 2022-2025  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

'''

OLD_HEADER = '''//
//  Copyright (C) 2022-2024  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

'''

source_files = glob.glob('./src/**/*.rs', recursive=True)
print('info: found', len(source_files), 'source files')

latest = 0
updating = 0
missing = 0

for source in source_files:
    data = ''
    with open(source, 'r') as fd:
        data = fd.read()
        # check if it starts with the latest header
        if data.startswith(NEW_HEADER) == True:
            latest += 1
            # print('info: source', source, 'already has latest header')
            continue
        # replace old header with new header
        elif data.startswith(OLD_HEADER) == True:
            updating += 1
            # print('info: source', source, 'is updating to latest header')
            data = data.replace(OLD_HEADER, NEW_HEADER)
        # write the new header at the beginning
        else:
            missing += 1
            # print('info: source', source, 'is missing latest header')
            data = NEW_HEADER + data
        pass
    # save the changes to the files
    with open(source, 'w') as fd:
        fd.write(data)
    pass

print('info: kept', latest, 'with latest license header')
print('info: updated', updating, 'to latest license header')
print('info: fixed', missing, 'with latest license header')