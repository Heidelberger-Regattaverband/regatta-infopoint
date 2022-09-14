sap.ui.define(function () {
  "use strict";

  var Formatter = {
    raceLabel: function (sRaceNumber, sRaceShortLabel, sComment) {
      return sRaceNumber + " - " + sRaceShortLabel + " " + sComment;
    }
  };

  return Formatter;
}, /* bExport= */ true);
