sap.ui.define(function () {
  "use strict";

  var Formatter = {
    stateLabel: function (iState, bCancelled) {

      if (bCancelled) {
        return "gestrichen";
      } else {
        switch (iState) {
          default:
            return "unbekannt";
          case 0:
            return "keine Daten";
          case 1:
            return "geplant";
          case 3:
            return "gestartet";
          case 4:
            return "offiziell";
          case 5:
            return "beendet";
        }
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
