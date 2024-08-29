# Vite
Its a Lite Vector database, everything lives in one file.
This is all a work in progress so some of this might not work.

## How to use
Creating a graph:
```
$ vlite new <path/filename> <initial vector> <m value> <m_max> <m_max0> <candidate list size>
```
This will save the graph in `path/filename.vlite` file.

Adding a vector:
```
$ vlite add <path/filename> <vector>
```
This will automagically insert your vector into the graph.

Searching a vector:
```
$ vlite search <path/filename> <search vector> <num results>
$ vlite search <path/filename> <id>
```
Will search a vector and return the matching vector, or id list

Features:
+ Inserting
+ Searching
+ Saving to file
+ Reading from file

Future features:
+ Editing file
+ Deleting
+ interpreter
+ speed
