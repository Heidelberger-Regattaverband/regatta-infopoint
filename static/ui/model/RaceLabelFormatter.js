sap.ui.define(function () {
  "use strict";

  var Formatter = {
    raceLabel: function (sRaceNumber, roundCode, divisionNumber) {

      switch (roundCode) {
        case "A":
          return sRaceNumber + " - Abteilung " + divisionNumber;
        case "R":
          return sRaceNumber + " - Hauptrennen";
        case "V":
          return sRaceNumber + " - Vorlauf " + divisionNumber;
        case "F":
          return sRaceNumber + " - Finale";
        default:
          return sRaceNumber;
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
