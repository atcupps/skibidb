# SkibiDB Working Design Doc

Author: [@atcupps](https://github.com/atcupps)  
This file can also be found [here](https://docs.google.com/document/d/154aUyTyEag_ZB08rwzDWC0yV-7S36UtLbFoSDfyln_A/edit?usp=sharing) as a Google Doc.

# Summary

This document serves to outline the design of various components and their interactions for SkibiDB. At this time, we plan for SkibiDB to be a relatively-simple DBMS; SkibiDB will exist both as a library with several crates that can be called directly from code as well as provide a binary that can be executed and configured which will accept and handle DDL and DML statements. SkibiDB will initially be built for only a single machine, but it may be expanded in the future to be parallelizable across several nodes.

## Design Philosophy

The current overarching goal is to create a DBMS that will run on a single machine, as well as a SQL dialect to access SkibiDB. Primarily, we aim to produce an end result that is:

* **Secure-by-default:** SkibiDB should ensure to the best of its ability that data is secure and unauthorized accesses are rejected without extra steps required of the database administrator; additionally, actions that are unsafe or likely to be highly destructive in nature should require confirmation by default. An emphasis should be placed on a defense-in-depth method and careful code analysis to reduce both the presence and impact of vulnerabilities.  
* **Developer-friendly:** Developers building software that relies on SkibiDB should enjoy the experience of using it. APIs should be intuitive and integrated into popular programming languages and frameworks and documentation should be comprehensive and up-to-date. Furthermore, the SQL dialect powering interaction with SkibiDB should be compact, consistent, and eliminate unnecessary complexities that make some existing SQL implementations difficult to work with.  
* **Single-machine optimized:** While SkibiDB is currently not planned to scale to multiple machines, it should take advantage of scalability within the machine by utilizing concurrency and/or parallelization, as well as optimizations in memory and cache usage, disk IO, etc.  
* **Progressive in features:** SkibiDB should take advantage of modern developments in programming language paradigms and computational abilities to get rid of anachronisms present in existing relational database implementations.

# DBMS Structure

SkibiDB will consist of the following general architecture design:

(The design diagram cannot be seen on GitHub; click [here](https://docs.google.com/document/d/154aUyTyEag_ZB08rwzDWC0yV-7S36UtLbFoSDfyln_A/edit?usp=sharing) to see the document on Google Docs.)

# System Flow

## DBA Controls

The Database Administrator (DBA) will have the ability to control the operation of the database before it runs (whether executed as a binary or called as a library) by using configuration files. When SkibiDB is actively running, the DBA can also execute commands via DDL to affect tables and schemas; additionally, the DBA can run SkibiDB-specific commands to affect various aspects of the DBMS.

In order to do this, the DBA will enter a command via the SkibiDB command-line interface, or can otherwise connect over a secure network connection to do the same. When a command is entered, it will be received by the Database Administration Toolkit System (DATS); DATS will check the DBA’s credentials via the Authorization Control Module (ACM). If the credentials supplied by the DBA allow for the change to be made as determined by the ACM, then the DBA will be able to modify the behavior of the:

* Statistical Calculations Operator: The DBA can specify certain statistics that should be tracked, or manually initiate recalculation of statistical properties  
* Cache Manager: The DBA can specify what should be present or not present within the DBMS Cache  
* Query Optimizer: The DBA can modify what optimizations can be made within the query processing unit  
* Index Manager: The DBA can destroy or create new indices, or modify the structure of indices  
* Authorization Control Module: The DBA can create new users, assign roles, and modify permissions

## Queries

A user (whether directly, through an API, or via a web server that connects to a SkibiDB instance) can query or modify the database by interacting with the Query Processing Unit (QPU). After the Authorization Control Module verifies that the credentials of the user allow the user to access the tables specified in the query, the Query Parser transforms the query (which is received as a string) into internal structs and other data forms; the QPU then optimizes the query based on statistics present within the DBMS cache, or which can be read from persistent statistics files if they are not present within the cache. The Query Evaluation Engine (QEE) then oversees execution of the query.

The QEE may use indexes via the Index Manager to expedite the processing of queries. When a read or write to disk must be made, the QEE does so through the Transaction Control System, which ensures ACID properties; and the Data Access and Storage System (DASS) which is responsible for actually interacting with the tables on disk. If tuples are added or deleted to tables, the DASS may also call the Index Manager to update any relevant persistent indexes. After all reads and writes are complete, the result will be returned back to the user.

## Statistical Calculations

It is useful to keep track of statistics regarding the tables present within the database in order to inform the Query Optimizer. The Statistical Calculations Operator (SCO) is responsible for actually calculating these statistics and writing the resulting information to persistent statistics files. Because calculating these statistics requires many reads from the database, these operations can be expensive; as a result, they are not done after every write to the database. Instead, statistics are calculated during times of low system load or otherwise in increments of time. After the SCO runs, it calls the Cache Manager to update the DBMS cache with new statistics.

# Components

Design documentation for individual components will be written here as they are planned and created.

## Data Access and Storage System (DASS)

The DASS is the lowest level of the DBMS aside from the actual data; the DASS directly reads from and writes to memory on an individual tuple level. The DASS supports both individual tuples as well as bulk reads and writes. The DASS does not take input from strings; instead, the QPU via the TCS supplies the DASS with internal data types that the DASS uses to operate on the disk memory. If an index is present and can be used for the existing operation, the DASS will also interact with the Index Manager to do so.

The DASS is not responsible for ensuring data access is authorized; this is done by the Authorization Control Module. DASS assumes all read and write requests it receives are authorized and valid.

When DASS receives a request to write or modify tuples, DASS is responsible for conducting integrity checks, removing duplicate tuples, and otherwise ensuring the validity of data being entered into the database.