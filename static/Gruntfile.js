module.exports = function (grunt) {

  // Project configuration.
  grunt.initConfig({
    // see: https://github.com/SAP/grunt-openui5#openui5_preload
    openui5_preload: {

      component: {
        options: {
          resources: {
            cwd: 'webapp',
            prefix: 'de/regatta_hd/infopoint'
          },
          dest: 'webapp'
        },
        components: 'de/regatta_hd/infopoint'
      }

    }
  });

  // Load the plugin that provides the "openui5_preload" task.
  grunt.loadNpmTasks('grunt-openui5');

  // Default task(s).
  grunt.registerTask('default', ['openui5_preload']);

};