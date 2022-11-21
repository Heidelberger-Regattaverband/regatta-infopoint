module.exports = function (grunt) {

  // Project configuration.
  grunt.initConfig({
    openui5_preload: {

      component: {
        options: {
          resources: {
            cwd: 'infoportal',
            prefix: 'de/regatta_hd/infopoint'
          },
          dest: 'infoportal'
        },
        components: 'de/regatta_hd/infopoint'
      }

    }
  });

  // Load the plugin that provides the "uglify" task.
  grunt.loadNpmTasks('grunt-openui5');

  // Default task(s).
  grunt.registerTask('default', ['openui5_preload']);

};