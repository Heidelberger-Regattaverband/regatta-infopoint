sap.ui.define(function () {
  "use strict";

  var Formatter = {
    heatLabel: function (sRoundCode, sRoundLabel) {
      switch (sRoundCode) {
        case "A":
          return "Abteilung " + sRoundLabel;
        case "R":
          return "Hauptrennen";
        case "V":
          return "Vorlauf " + sRoundLabel;
        case "F":
          return "Finale";
        default:
          return "";
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
