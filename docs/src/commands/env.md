# __orbit env__

## __NAME__

env - print orbit environment information

## __SYNOPSIS__

```
orbit env [options]
```

## __DESCRIPTION__

Displays environment variables as key-value pairs related to Orbit.

By default, this command prints information as a shell script. If one or more
variable names are given as arguments using `<key>`, then it will print the 
value of each provided variable on its own line.

Environment information can change based on where the command is executed.

Environment variables that are known only at runtime are not displayed. Be
sure to review the documentation for a list of all environment variables set 
by Orbit.

## __OPTIONS__

`<key>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display this variable's value

## __EXAMPLES__

```
orbit env
orbit env ORBIT_HOME
orbit env ORBIT_MANIFEST_DIR ORBIT_IP_NAME
```

