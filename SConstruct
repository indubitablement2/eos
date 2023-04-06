env = SConscript('godot-cpp/SConstruct')

env.Tool("compilation_db")
cdb = env.CompilationDatabase()

env.Append(CPPPATH='src/')

outputpath = 'godot/core/lib/gdext' + env['SHLIBSUFFIX']
src = Glob('src/*.cpp')
sharedlib = env.SharedLibrary(outputpath, src)

Default([sharedlib, cdb])
# Default(sharedlib)

# if env['platform'] == 'windows':
#     sharedlib = env.SharedLibrary(outputpath, src)
# elif env['platform'] == 'linux':
#     sharedlib = env.SharedLibrary(outputpath, src)
#     Default(sharedlib)
