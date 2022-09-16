sap.ui.define(function () {
  "use strict";

  var Formatter = {
    boatLabel: function (sShortLabel, iBoatNumber) {
      if (iBoatNumber > 0){
        return sShortLabel + " - Boot " + iBoatNumber;  
      }
      return sShortLabel;  
    },

    raceLabel: function (sRaceNumber, sRaceShortLabel, sComment) {
      return sRaceNumber + " - " + sRaceShortLabel + " " + sComment;
    }

  };

  return Formatter;
}, /* bExport= */ true);
