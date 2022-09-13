sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "../model/StateLabelFormatter",
  "../model/RaceLabelFormatter"
], function (Controller, JSONModel, StateLabelFormatter, RaceLabelFormatter) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.HeatsTable", {
    stateLabelFormatter: StateLabelFormatter
  });

});