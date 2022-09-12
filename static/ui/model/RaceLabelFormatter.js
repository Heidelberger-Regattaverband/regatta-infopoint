sap.ui.define(function () {
  "use strict";

  var Formatter = {
    raceLabel: function (sRaceNumber, roundCode, divisionNumber) {
      if (roundCode == "A") {
        return sRaceNumber + " - Abteilung " + divisionNumber;
      } else if (roundCode == "R") {
        return sRaceNumber + " - Hauptrennen";
      } else {
        return sRaceNumber;
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
