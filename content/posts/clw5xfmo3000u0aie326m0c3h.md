---
title: "How to extract all TODOs from code using Scala-CLI"
datePublished: Tue May 14 2024 05:02:48 GMT+0000 (Coordinated Universal Time)
cuid: clw5xfmo3000u0aie326m0c3h
slug: how-to-extract-all-todos-from-code-using-scala-cli
cover: https://cdn.hashnode.com/res/hashnode/image/upload/v1715588640335/b44b5f8b-fd0f-4e23-9145-e048b7ebe7db.png
tags: scala, scala-cli

---

In recent years, [Scala CLI](https://scala-cli.virtuslab.org) has replaced sbt for my home and smaller work projects. Scala CLI lacks the extensive plugin ecosystem of sbt, so you need to write any additional functionalities yourself. Fortunately, writing simple scripts is a primary use case for Scala CLI, which we'll leverage here to create the needed functionality.

What I needed was a straightforward method to list all 'TO DO's and related comments in the code.

Specifically the functionality is to list comments looking like:

```scala
// TODO: A to do I want to keep on my rader.
// FUN: Some small tyding I could to improve the code in lost time.
// FIXME: A probably minor thing to fix.
// XXX: Anything else I do not want to forget about.
```

Here's the full code. I'll focus on the key aspects of the functionality and how to run the script.

```scala
/** Find all TODOs and related tags in the source code of a project.
 * To run using Scala CLI: `scala-cli run <this script>`
 **/

//> using scala 3.4.1
//> using dep com.lihaoyi::os-lib:0.10.0

import os._
import scala.io.Source
import scala.util.matching.Regex

// Change relative path to the directory containing the source code
val todos = extractTodosFromDirectory("src")
todos.foreach(println)

def extractTodosFromFile(filePath: Path): List[String] = 
  val source = Source.fromFile(filePath.toString)
  val fileContent = try source.mkString finally source.close()

  val fileName = filePath.last

  val todoPattern: Regex = """//(\s)*(TODO|FUN|XXX|FIXME): (.*)""".r
  val todos = todoPattern.findAllIn(fileContent).matchData.map{m =>
    val tag = m.group(2)
    val description = m.group(3)
    s"[$tag] $description ($fileName) "
  }.toList

  todos
end extractTodosFromFile

def extractTodosFromDirectory(directoryPath: String): List[String] = 
  val dir = os.pwd /directoryPath
  val files = os.list(dir).filter(os.isFile(_))
  files.flatMap(file => extractTodosFromFile(file)).toList
end extractTodosFromDirectory
```

### Noteworthy about the script

To run the script, assuming you've named it `todo-list.sc`:

```bash
scala-cli run todo-list.sc
```

The script scans the `src` directory, located one level below the script. Change this to your desired location or modify the script to take this location as an argument.

```scala
val todos = extractTodosFromDirectory("src")
```

The script searches for the following tags: `TODO|FUN|XXX|FIXME`

```scala
val todoPattern: Regex = """//(\s)*(TODO|FUN|XXX|FIXME): (.*)""".r
```

### Sidebar: custom code or plugins?

For simple tasks, writing and maintaining your own code is often easier than managing a plugin:

* A plugin requires some understanding and configuration.
    
* A plugin may not meet your needs exactly, for example, also scanning for a 'FUN' tag.
    
* A plugin needs updating.
    
* A plugin can become unmaintained.
    
* A plugin introduces third-party code, which may not always be secure.
    

Writing this custom code took less than 30 minutes, including a diversion into exploring scala-native. Once functional, it seldom needs updates. The main downside is its dependency on the `os-lib`library, but that's a compromise I accept to avoid spending more time on this script versus managing the library. If necessary, removing this dependency is straightforward.

Thank you for reading!

Hans